import { writable, derived, get } from "svelte/store";
import type { PlayerConfig, PlayerStatus, LogLine } from "../../types";
import { api, onPlayerLog, onPlayerStatus } from "../api";

export const players = writable<PlayerConfig[]>([]);
export const selectedId = writable<string | null>(null);
export const statuses = writable<Record<string, PlayerStatus>>({});
export const logs = writable<Record<string, LogLine[]>>({});

export const selectedPlayer = derived(
  [players, selectedId],
  ([$players, $id]) => $players.find((p) => p.id === $id) ?? null
);

export async function refreshPlayers() {
  const list = await api.listPlayers();
  players.set(list);
  if (!get(selectedId) && list.length > 0) selectedId.set(list[0].id);
  if (get(selectedId) && !list.some((p) => p.id === get(selectedId))) {
    selectedId.set(list[0]?.id ?? null);
  }
  // Query each player's current status from the backend so the UI is never stale.
  // Events handle live updates, but this covers the initial load and reconnect gaps.
  for (const p of list) {
    try {
      const s = await api.getPlayerStatus(p.id);
      statuses.update((prev) => ({ ...prev, [p.id]: s }));
    } catch { /* ignore per-player errors */ }
  }
}

export async function addPlayer() {
  const name = `Player ${Math.floor(Math.random() * 1000)}`;
  const p: PlayerConfig = {
    id: crypto.randomUUID(),
    name,
    account: { kind: "offline", username: "Steve" },
    server: { host: "localhost", port: 25565 },
    version_override: null,
    on_join_commands: [],
    on_join_delay_secs: 0,
    facing: null,
    behaviors_csharp: defaultBehaviorTemplate(),
    force_quit: {
      on_totem_pop: false,
      on_attacked: false,
      on_death: false,
      low_health: null,
      chat_regex: null,
    },
    auto_totem: false,
    eat_when_hungry: false,
    auto_reconnect: {
      enabled: true,
      retries: -1,
      delay_min: 8,
      delay_max: 60,
      kick_messages: [
        "Connection has been lost",
        "Server is restarting",
        "Server is full",
      ],
    },
  };
  await api.addPlayer(p);
  await refreshPlayers();
  selectedId.set(p.id);
}

export async function savePlayer(p: PlayerConfig) {
  await api.updatePlayer(p);
  await refreshPlayers();
}

export async function deletePlayer(id: string) {
  await api.deletePlayer(id);
  await refreshPlayers();
}

export async function startPlayer(id: string) {
  const player = get(players).find((p) => p.id === id);
  const playerName = player?.name ?? id;
  statuses.update((s) => ({ ...s, [id]: { kind: "starting" } }));
  try {
    console.log(`[startPlayer] Starting player "${playerName}" (${id})...`);
    await api.startPlayer(id);
    console.log(`[startPlayer] Player "${playerName}" (${id}) started successfully.`);
    await subscribeToPlayer(id);
  } catch (e) {
    const reason = String(e);
    console.error(`[startPlayer] FAILED to start player "${playerName}" (${id}):`, reason, e);
    statuses.update((s) => ({ ...s, [id]: { kind: "failed", reason } }));
  }
}

export async function stopPlayer(id: string) {
  try {
    await api.stopPlayer(id);
  } catch (e) {
    console.error(e);
  }
}

export async function sendCommand(id: string, command: string) {
  await api.sendCommand(id, command);
  logs.update((l) => ({
    ...l,
    [id]: [
      ...(l[id] ?? []),
      { ts: Date.now(), stream: "stdout", text: `> ${command}` },
    ],
  }));
}

const unlisteners: Record<string, (() => void)[]> = {};

export async function subscribeToPlayer(id: string) {
  if (unlisteners[id]) return; // already subscribed
  unlisteners[id] = [];

  // Await both listeners so they're definitely registered before we return.
  const [unLog, unStatus] = await Promise.all([
    onPlayerLog(id, (line) => {
      logs.update((l) => {
        const arr = l[id] ?? [];
        const next = [...arr, line];
        if (next.length > 2000) next.splice(0, next.length - 2000);
        return { ...l, [id]: next };
      });
    }),
    onPlayerStatus(id, (status) => {
      console.log(`[subscribeToPlayer] Status event for ${id}:`, JSON.stringify(status));
      statuses.update((s) => ({ ...s, [id]: status }));
    }),
  ]);
  unlisteners[id].push(unLog, unStatus);

  // Query the backend for the player's current status so the UI is never
  // stuck on "stopped" for a player that's already running.
  try {
    const current = await api.getPlayerStatus(id);
    statuses.update((s) => {
      // Only apply if we haven't already received a more-specific status event.
      if (!s[id] || s[id].kind === "stopped") {
        return { ...s, [id]: current };
      }
      return s;
    });
  } catch {
    // If the query fails (e.g. player not in registry), leave the store as-is.
  }
}

export function statusLabel(s: PlayerStatus | undefined): string {
  if (!s) return "stopped";
  switch (s.kind) {
    case "stopped": return "stopped";
    case "starting": return "starting";
    case "connected": return "connected";
    case "reconnecting": return "reconnecting";
    case "failed": return "failed";
  }
}

export function defaultBehaviorTemplate(): string {
  return `//MCCScript 1.0
// Leo's AFK Helper - behavior script. Edit freely. Runs inside MCC at ~20 ticks/sec.
// Config-driven features (facing, on-join commands, auto-totem, force-quit) are applied
// automatically by the generated bot. Override the OnUser* hooks below to add behavior.
MCC.LoadBot(new AfkBot());

//MCCScript Extensions
using System;
using System.Text.RegularExpressions;
using MinecraftClient;
using MinecraftClient.Scripting;
using MinecraftClient.Mapping;
using MinecraftClient.Inventory;

public class AfkBot : ChatBot
{
    // Called once after each server join (after on-join commands and facing are applied).
    public virtual void OnUserJoin() { }

    // Called ~20 times per second. Keep it cheap; never block.
    // Example attack loop (uncomment):
    //   if (!ClientIsMoving())
    //   {
    //       var target = Game.FindNearestEntity(typeFilter: "mob", radius: 4.0);
    //       if (target.Ok && target.Data.Entity != null)
    //       {
    //           LookAtLocation(target.Data.Entity.Location);
    //           InteractEntity(target.Data.Entity.ID, InteractType.Attack);
    //       }
    //   }
    public virtual void OnUserUpdate() { }

    // Called when a chat/kick line arrives (verbatim text, color codes stripped).
    public virtual void OnUserChat(string text) { }
}
`;
}
