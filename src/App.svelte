<script lang="ts">
  import { onMount } from 'svelte';
  import PlayerList from './lib/components/PlayerList.svelte';
  import PlayerEditor from './lib/components/PlayerEditor.svelte';
  import DeviceCodePrompt from './lib/components/DeviceCodePrompt.svelte';
  import { players, statuses, selectedPlayer, refreshPlayers } from './lib/stores/players';
  import type { DeviceCodePrompt as DC } from './types';

  let devicePrompt: { id: string; prompt: DC } | null = null;
  let theme: 'dark' | 'light' = 'dark';

  function handleDeviceCode(id: string, prompt: DC) {
    devicePrompt = { id, prompt };
  }

  function toggleTheme() {
    theme = theme === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('leos-mc-theme', theme);
  }

  onMount(() => {
    const saved = localStorage.getItem('leos-mc-theme') as 'dark' | 'light' | null;
    if (saved) {
      theme = saved;
      document.documentElement.setAttribute('data-theme', theme);
    }
    refreshPlayers();
  });

  $: devicePromptPlayerName = (() => {
    if (!devicePrompt) return '';
    const id = devicePrompt.id;
    return $players.find((p) => p.id === id)?.name ?? 'Unknown';
  })();

  // Live count of players that are actively running (connected/starting/reconnecting).
  $: runningCount = $players.filter((p) => {
    const s = $statuses[p.id]?.kind;
    return s && s !== 'stopped' && s !== 'failed';
  }).length;
</script>

<main class="app-shell">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="brand">
        <div class="brand-logo" aria-hidden="true">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="2" width="20" height="20" rx="5"/>
            <rect x="6" y="6" width="12" height="12" rx="2"/>
            <line x1="8" y1="12" x2="16" y2="12"/>
            <line x1="12" y1="8" x2="12" y2="16"/>
          </svg>
        </div>
        <div class="brand-text">
          <span class="brand-name">AFK Helper</span>
          <span class="brand-version">MCC v476</span>
        </div>
      </div>
      <button class="theme-toggle-btn" on:click={toggleTheme} title="Toggle theme" aria-label="Toggle light or dark theme">
        {#if theme === 'dark'}
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
          </svg>
        {:else}
          <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        {/if}
      </button>
    </div>

    <!-- Live fleet summary -->
    <div class="fleet-summary" class:active={runningCount > 0}>
      <span class="fleet-dot"></span>
      <span class="fleet-text">
        {#if runningCount > 0}
          <strong class="nums">{runningCount}</strong> running
        {:else}
          No bots running
        {/if}
      </span>
    </div>

    <PlayerList deviceCodeHandler={handleDeviceCode} />
  </aside>

  <!-- Main panel -->
  <section class="main-content">
    {#if $selectedPlayer}
      <div class="editor-view animate-fade">
        <PlayerEditor player={$selectedPlayer} />
      </div>
    {:else}
      <div class="welcome animate-fade-up">
        <div class="welcome-icon" aria-hidden="true">
          <svg width="56" height="56" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="2" width="20" height="20" rx="5"/>
            <rect x="6" y="6" width="12" height="12" rx="2"/>
            <circle cx="12" cy="12" r="2"/>
          </svg>
        </div>
        <h1 class="welcome-title">Leo's Minecraft AFK Helper</h1>
        <p class="welcome-desc">
          Create a player on the left, configure its server, account, and behavior, then launch it.
        </p>
        <div class="version-chip">
          <span class="version-dot"></span>
          Minecraft Java 1.4.6 &ndash; 26.1 &middot; including 1.21.11
        </div>
      </div>
    {/if}
  </section>
</main>

<DeviceCodePrompt
  prompt={devicePrompt?.prompt ?? null}
  playerName={devicePromptPlayerName}
/>

<style>
  .app-shell {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  /* ═══════ SIDEBAR ═══════ */
  .sidebar {
    width: var(--sidebar-w);
    flex-shrink: 0;
    background: var(--bg-base);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-4) var(--space-4) var(--space-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .brand-logo {
    width: 38px;
    height: 38px;
    background: linear-gradient(135deg, var(--accent-strong), var(--accent));
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    flex-shrink: 0;
    box-shadow: 0 4px 14px var(--accent-glow-strong);
  }

  .brand-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .brand-name {
    font-size: 14.5px;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.02em;
    line-height: 1.2;
  }

  .brand-version {
    font-size: 10.5px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
    letter-spacing: 0.03em;
  }

  /* ═══════ FLEET SUMMARY ═══════ */
  .fleet-summary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin: var(--space-3) var(--space-4) var(--space-1);
    padding: 7px 12px;
    background: var(--bg-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    font-size: 11.5px;
    color: var(--text-tertiary);
    font-weight: 500;
    flex-shrink: 0;
  }

  .fleet-summary.active {
    border-color: var(--border-accent);
    background: linear-gradient(135deg, var(--accent-glow), var(--bg-raised));
    color: var(--text-secondary);
  }

  .fleet-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--text-tertiary);
    flex-shrink: 0;
  }

  .fleet-summary.active .fleet-dot {
    background: var(--green-icon);
    box-shadow: 0 0 8px var(--green);
    animation: badgePulse 1.6s ease-in-out infinite;
  }

  .fleet-text strong {
    color: var(--accent);
    font-weight: 700;
  }

  /* ═══════ MAIN ═══════ */
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    height: 100%;
    overflow: hidden;
    background: var(--bg-root);
  }

  .editor-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* ═══════ WELCOME ═══════ */
  .welcome {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: var(--space-12);
    gap: var(--space-4);
  }

  .welcome-icon {
    width: 92px;
    height: 92px;
    background: linear-gradient(135deg, var(--accent-strong), var(--accent));
    border-radius: var(--radius-xl);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    box-shadow: var(--shadow-glow-green);
    margin-bottom: var(--space-2);
  }

  .welcome-title {
    font-size: 30px;
    font-weight: 800;
    color: var(--text-primary);
    letter-spacing: -0.035em;
  }

  .welcome-desc {
    max-width: 460px;
    font-size: 14px;
    color: var(--text-secondary);
    line-height: 1.6;
  }

  .version-chip {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-1) 12px;
    background: var(--bg-raised);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: var(--space-1);
  }

  .version-dot {
    width: 7px;
    height: 7px;
    background: var(--green-icon);
    border-radius: 50%;
    box-shadow: 0 0 8px var(--green);
  }
</style>
