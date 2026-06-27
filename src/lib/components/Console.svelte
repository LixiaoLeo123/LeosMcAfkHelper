<script lang="ts">
  import { logs, sendCommand } from '../stores/players';

  export let playerId: string;

  let input = '';
  let scroller: HTMLDivElement;

  $: if ($logs[playerId]) {
    $logs[playerId].length;
    requestAnimationFrame(() => {
      if (scroller) {
        const threshold = 50;
        const atBottom = scroller.scrollHeight - scroller.scrollTop - scroller.clientHeight < threshold;
        if (atBottom) scroller.scrollTop = scroller.scrollHeight;
      }
    });
  }

  function submit() {
    const cmd = input.trim();
    if (!cmd) return;
    sendCommand(playerId, cmd);
    input = '';
  }
</script>

<div class="console">
  <div class="term-header">
    <div class="term-dots" aria-hidden="true">
      <span class="t-dot red"></span>
      <span class="t-dot amber"></span>
      <span class="t-dot green"></span>
    </div>
    <span class="term-title">mcc@{playerId}</span>
    <span class="term-meta nums">{($logs[playerId] ?? []).length} lines</span>
  </div>

  <div class="term-body" bind:this={scroller}>
    {#each ($logs[playerId] ?? []) as line, i (i)}
      <div class="log-line {line.stream}">
        <span class="log-ts nums">{new Date(line.ts).toLocaleTimeString()}</span>
        <span class="log-sep"></span>
        <span class="log-text">{line.text}</span>
      </div>
    {/each}

    {#if !($logs[playerId]?.length)}
      <div class="term-empty">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="4 17 10 11 4 5"/>
          <line x1="12" y1="19" x2="20" y2="19"/>
        </svg>
        <p>No output yet</p>
        <span>Start the player to see MCC logs here</span>
      </div>
    {/if}
  </div>

  <div class="term-input">
    <span class="prompt-arrow" aria-hidden="true">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="9 18 15 12 9 6"/>
      </svg>
    </span>
    <form on:submit|preventDefault={submit} style="flex:1;display:flex;gap:8px;">
      <input
        type="text"
        placeholder="Send command or chat (/help, script, reco, quit)…"
        bind:value={input}
        style="flex:1"
        aria-label="Send command or chat to this player"
      />
      <button type="submit">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <line x1="22" y1="2" x2="11" y2="13"/>
          <polygon points="22 2 15 22 11 13 2 9 22 2"/>
        </svg>
        Send
      </button>
    </form>
  </div>
</div>

<style>
  .console {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--terminal-bg);
    border-radius: var(--radius-lg);
    border: 1px solid var(--terminal-border);
    margin: var(--space-2) var(--space-4) var(--space-4);
    overflow: hidden;
    box-shadow: var(--shadow-md);
  }

  .term-header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--terminal-header-bg);
    border-bottom: 1px solid var(--terminal-border);
    user-select: none;
  }

  .term-dots {
    display: flex;
    gap: 7px;
    flex-shrink: 0;
  }

  .t-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .t-dot.red { background: #ef4444; }
  .t-dot.amber { background: #f59e0b; }
  .t-dot.green { background: #22c55e; }

  .term-title {
    font-size: 11.5px;
    font-weight: 500;
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .term-meta {
    margin-left: auto;
    font-size: 10.5px;
    color: var(--text-tertiary);
    font-family: var(--font-mono);
  }

  .term-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: var(--space-2) var(--space-3);
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.7;
    color: var(--terminal-text);
    background: var(--terminal-bg);
  }

  .log-line {
    display: flex;
    gap: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .log-ts {
    color: var(--text-tertiary);
    flex-shrink: 0;
    margin-right: var(--space-2);
    opacity: 0.55;
    font-size: 11px;
  }

  .log-sep {
    width: 2px;
    background: var(--border-subtle);
    margin: 2px var(--space-2) 2px 0;
    flex-shrink: 0;
  }

  .log-text {
    color: var(--terminal-text);
  }

  .log-line.stderr .log-text {
    color: var(--red);
  }

  .log-line.stderr .log-sep {
    background: var(--red);
    opacity: 0.35;
  }

  .term-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--space-2);
    color: var(--text-tertiary);
    opacity: 0.55;
  }

  .term-empty p {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .term-empty span {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .term-input {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--terminal-header-bg);
    border-top: 1px solid var(--terminal-border);
  }

  .prompt-arrow {
    color: var(--accent);
    display: inline-flex;
    flex-shrink: 0;
  }

  .term-input input {
    background: var(--terminal-bg);
    color: var(--terminal-text);
    font-family: var(--font-mono);
    font-size: 12.5px;
    border-color: var(--terminal-border);
  }

  .term-input input::placeholder {
    color: var(--text-tertiary);
  }

  .term-input input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px var(--accent-glow);
  }

  .term-input button {
    flex-shrink: 0;
  }
</style>
