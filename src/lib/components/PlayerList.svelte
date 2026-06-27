<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    players, selectedId, statuses, statusLabel,
    addPlayer, deletePlayer, startPlayer, stopPlayer, subscribeToPlayer
  } from '../stores/players';
  import { onDeviceCode } from '../api';

  export let deviceCodeHandler: (id: string, prompt: { code: string; url: string }) => void;

  let deviceCodeCleanupFns: (() => void)[] = [];
  let hoveredId: string | null = null;

  $: $players.forEach((p) => subscribeToPlayer(p.id));

  $: {
    deviceCodeCleanupFns.forEach((fn) => fn());
    deviceCodeCleanupFns = [];
    $players.forEach((p) => {
      onDeviceCode(p.id, (prompt) => deviceCodeHandler(p.id, prompt)).then((un) => {
        deviceCodeCleanupFns.push(un);
      });
    });
  }

  onDestroy(() => deviceCodeCleanupFns.forEach((fn) => fn()));

  $: statusLabels = (() => {
    const labels: Record<string, string> = {};
    for (const p of $players) labels[p.id] = statusLabel($statuses[p.id]);
    return labels;
  })();

  $: statusTitles = (() => {
    const titles: Record<string, string> = {};
    for (const p of $players) {
      const s = $statuses[p.id];
      titles[p.id] = s?.kind === 'failed' ? s.reason : '';
    }
    return titles;
  })();

  $: isRunning = (() => {
    const map: Record<string, boolean> = {};
    for (const p of $players) {
      const k = $statuses[p.id]?.kind;
      map[p.id] = k === 'starting' || k === 'connected' || k === 'reconnecting';
    }
    return map;
  })();

  async function onDelete(id: string, name: string) {
    if (confirm(`Delete "${name}"? This will stop it if running and remove it permanently.`)) {
      await deletePlayer(id);
    }
  }

  function accountIconKind(kind: string): string {
    return kind === 'microsoft' ? 'ms' : 'offline';
  }
</script>

<div class="player-list">
  <div class="list-header">
    <span class="list-title">Players</span>
    <span class="player-count nums">{$players.length}</span>
  </div>

  <div class="list-body">
    {#if $players.length === 0}
      <div class="empty-state animate-fade-up">
        <div class="empty-art" aria-hidden="true">
          <svg width="46" height="46" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
          </svg>
        </div>
        <p class="empty-text">No players configured</p>
        <button class="primary" on:click={addPlayer}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
          </svg>
          Create your first player
        </button>
      </div>
    {:else}
      <div class="card-list">
        {#each $players as p, idx (p.id)}
          <div
            class="player-card animate-slide"
            class:selected={p.id === $selectedId}
            class:running={isRunning[p.id]}
            style="animation-delay: {idx * 0.04}s"
            role="listitem"
            on:mouseenter={() => (hoveredId = p.id)}
            on:mouseleave={() => (hoveredId = null)}
          >
            <!-- status accent rail (color + position, not color alone) -->
            <span class="card-rail {statusLabels[p.id] || 'stopped'}" aria-hidden="true"></span>

            <button class="card-main" on:click={() => selectedId.set(p.id)} aria-label="Select player {p.name}">
              <div class="card-left">
                {#if accountIconKind(p.account.kind) === 'ms'}
                  <svg class="card-avatar" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                    <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                  </svg>
                {:else}
                  <svg class="card-avatar" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                    <circle cx="12" cy="7" r="4"/>
                  </svg>
                {/if}
              </div>
              <div class="card-info">
                <span class="card-name">{p.name}</span>
                <span class="card-server">{p.server.host}{p.server.port ? ':' + p.server.port : ''}</span>
              </div>
              <span
                class="badge {statusLabels[p.id] || 'stopped'}"
                title={statusTitles[p.id] || ''}
              >
                {statusLabels[p.id] || 'stopped'}
              </span>
            </button>

            <!-- Actions: visible on selected + hover, so they're never hover-only -->
            {#if p.id === $selectedId || hoveredId === p.id}
              <div class="card-actions animate-fade">
                <button
                  class="action-btn play"
                  on:click={() => startPlayer(p.id)}
                  title="Start player"
                  aria-label="Start {p.name}"
                >
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
                    <polygon points="5,3 19,12 5,21"/>
                  </svg>
                </button>
                <button
                  class="action-btn stop"
                  on:click={() => stopPlayer(p.id)}
                  title="Stop player"
                  aria-label="Stop {p.name}"
                >
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none" aria-hidden="true">
                    <rect x="4" y="4" width="16" height="16" rx="2"/>
                  </svg>
                </button>
                <button
                  class="action-btn del"
                  on:click={() => onDelete(p.id, p.name)}
                  title="Delete player"
                  aria-label="Delete {p.name}"
                >
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                    <line x1="18" y1="6" x2="6" y2="18"/>
                    <line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <div class="list-footer">
    <button class="primary add-btn" on:click={addPlayer}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
        <line x1="12" y1="5" x2="12" y2="19"/>
        <line x1="5" y1="12" x2="19" y2="12"/>
      </svg>
      Add Player
    </button>
  </div>
</div>

<style>
  .player-list {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--space-3) var(--space-4) var(--space-1);
  }

  .list-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-tertiary);
  }

  .player-count {
    font-size: 11px;
    font-weight: 700;
    background: var(--bg-hover);
    color: var(--text-secondary);
    padding: 2px 9px;
    border-radius: var(--radius-full);
    min-width: 24px;
    text-align: center;
  }

  .list-body {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-1) var(--space-2) var(--space-2);
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-8) var(--space-4);
    text-align: center;
  }

  .empty-art {
    color: var(--text-tertiary);
    opacity: 0.45;
  }

  .empty-text {
    font-size: 13px;
    color: var(--text-secondary);
  }

  /* ── Cards ── */
  .card-list {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .player-card {
    position: relative;
    background: var(--bg-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
    transition: border-color var(--dur-fast) var(--ease-in-out),
                box-shadow var(--dur-fast) var(--ease-in-out),
                transform var(--dur-fast) var(--ease-in-out);
  }

  .player-card:hover {
    border-color: var(--border-default);
    box-shadow: var(--shadow-sm);
  }

  .player-card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent), 0 8px 24px var(--accent-glow-strong);
  }

  /* status accent rail down the left edge */
  .card-rail {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 3px;
    background: var(--text-tertiary);
    opacity: 0.5;
    transition: opacity var(--dur-fast) var(--ease-in-out);
  }

  .player-card.running .card-rail { opacity: 1; }
  .player-card.running .card-rail.connected { background: var(--green); box-shadow: 0 0 12px var(--green); }
  .player-card.running .card-rail.starting { background: var(--amber); }
  .player-card.running .card-rail.reconnecting { background: var(--amber); }
  .card-rail.failed { background: var(--red); opacity: 0.8; }

  .card-main {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3) var(--space-2) var(--space-4);
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    font-size: inherit;
    border-radius: 0;
  }

  .card-main:hover {
    background: transparent;
    transform: none;
  }

  .card-main:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }

  .card-left {
    flex-shrink: 0;
  }

  .card-avatar {
    width: 32px;
    height: 32px;
    padding: 7px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: color var(--dur-fast) var(--ease-in-out);
  }

  .player-card.selected .card-avatar {
    color: var(--accent);
  }

  .card-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .card-name {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .player-card.selected .card-name {
    color: var(--accent);
  }

  .card-server {
    font-size: 11px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* ── Hover/Selected Actions ── */
  .card-actions {
    display: flex;
    gap: 5px;
    padding: 0 var(--space-3) var(--space-2) var(--space-4);
  }

  .action-btn {
    flex: 1;
    padding: 6px 0;
    border-radius: var(--radius-xs);
    font-size: 12px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-hover);
    color: var(--text-secondary);
    height: 30px;
  }

  .action-btn:hover:not(:disabled) {
    background: var(--bg-active);
    color: var(--text-primary);
  }

  .action-btn.play:hover {
    border-color: var(--green);
    color: var(--green);
    background: var(--green-glow);
  }

  .action-btn.stop:hover {
    border-color: var(--amber);
    color: var(--amber);
    background: var(--amber-glow);
  }

  .action-btn.del:hover {
    border-color: var(--red);
    color: var(--red);
    background: var(--red-glow);
  }

  /* ── Footer ── */
  .list-footer {
    padding: var(--space-2) var(--space-3) var(--space-3);
    border-top: 1px solid var(--border-subtle);
  }

  .add-btn {
    width: 100%;
    padding: var(--space-2);
    font-size: 13px;
    font-weight: 600;
  }
</style>
