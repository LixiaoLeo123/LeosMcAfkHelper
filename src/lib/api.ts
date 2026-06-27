import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  PlayerConfig,
  PlayerStatus,
  LogLine,
  DeviceCodePrompt,
} from "../types";

// ---- Command wrappers (invoke -> Rust) ----

export const api = {
  listPlayers: (): Promise<PlayerConfig[]> => invoke("list_players"),
  addPlayer: (p: PlayerConfig): Promise<void> => invoke("add_player", { player: p }),
  updatePlayer: (p: PlayerConfig): Promise<void> => invoke("update_player", { player: p }),
  deletePlayer: (id: string): Promise<void> => invoke("delete_player", { id }),
  startPlayer: (id: string): Promise<void> => invoke("start_player_cmd", { id }),
  stopPlayer: (id: string): Promise<void> => invoke("stop_player_cmd", { id }),
  sendCommand: (id: string, command: string): Promise<void> =>
    invoke("send_command_cmd", { id, command }),
  getPlayerStatus: (id: string): Promise<PlayerStatus> =>
    invoke("get_player_status", { id }),
  getRecentLogs: (id: string): Promise<LogLine[]> =>
    invoke("get_recent_logs", { id }),
  previewScript: (p: PlayerConfig): Promise<string> =>
    invoke("preview_script", { player: p }),
  openPlayerWorkdir: (id: string): Promise<void> =>
    invoke("open_player_workdir", { id }),
};

// ---- Event subscriptions (Rust -> frontend) ----

export function onPlayerLog(
  id: string,
  cb: (line: LogLine) => void
): Promise<UnlistenFn> {
  return listen<LogLine>(`player:${id}:log`, (e) => cb(e.payload));
}

export function onPlayerStatus(
  id: string,
  cb: (status: PlayerStatus) => void
): Promise<UnlistenFn> {
  return listen<PlayerStatus>(`player:${id}:status`, (e) => cb(e.payload));
}

export function onDeviceCode(
  id: string,
  cb: (prompt: DeviceCodePrompt) => void
): Promise<UnlistenFn> {
  return listen<DeviceCodePrompt>(`player:${id}:devicecode`, (e) =>
    cb(e.payload)
  );
}
