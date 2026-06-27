<script lang="ts">
  import type { PlayerConfig } from '../../types';
  import { savePlayer } from '../stores/players';
  import { api } from '../api';
  import CodeEditor from './CodeEditor.svelte';
  import Console from './Console.svelte';

  export let player: PlayerConfig;

  let local: PlayerConfig = JSON.parse(JSON.stringify(player));
  let tab: 'config' | 'behavior' | 'console' = 'config';
  let preview = '';
  let dirty = false;
  let saving = false;
  let savedMsg = '';
  let savedOk = false;

  let sections = {
    server: true,
    account: true,
    onjoin: false,
    facing: false,
    reconnect: false,
    forcequit: false
  };

  function toggleSection(key: keyof typeof sections) {
    sections[key] = !sections[key];
    sections = { ...sections };
  }

  $: if (player.id !== local.id) {
    local = JSON.parse(JSON.stringify(player));
    dirty = false;
  }

  $: if (player) {
    local.id = player.id;
  }

  function markDirty() {
    dirty = true;
    savedMsg = '';
  }

  async function save() {
    saving = true;
    savedMsg = '';
    try {
      await savePlayer(local);
      dirty = false;
      savedOk = true;
      savedMsg = 'Saved';
      setTimeout(() => (savedMsg = ''), 2200);
    } catch (e) {
      savedOk = false;
      savedMsg = 'Error: ' + String(e);
    } finally {
      saving = false;
    }
  }

  async function refreshPreview() {
    try {
      preview = await api.previewScript(local);
    } catch (e) {
      preview = '// Error generating preview: ' + String(e);
    }
  }

  $: if (tab === 'behavior' && !preview) refreshPreview();

  async function openWorkdir() {
    try {
      await api.openPlayerWorkdir(local.id);
    } catch (e) {
      alert(String(e));
    }
  }

  function setAccountKind(kind: 'offline' | 'microsoft') {
    if (kind === local.account.kind) return;
    local.account = kind === 'microsoft'
      ? { kind: 'microsoft', login: local.account.kind === 'offline' ? local.account.username + '@example.com' : local.account.login }
      : { kind: 'offline', username: local.account.kind === 'microsoft' ? 'Steve' : local.account.username };
    markDirty();
  }

  function addJoinCmd() {
    local.on_join_commands = [...local.on_join_commands, '/'];
    markDirty();
  }
  function updateJoinCmd(i: number, v: string) {
    local.on_join_commands[i] = v;
    local = { ...local };
    markDirty();
  }
  function removeJoinCmd(i: number) {
    local.on_join_commands = local.on_join_commands.filter((_, idx) => idx !== i);
    markDirty();
  }

  function addKickMsg() {
    local.auto_reconnect.kick_messages = [...local.auto_reconnect.kick_messages, ''];
    markDirty();
  }
  function updateKickMsg(i: number, v: string) {
    local.auto_reconnect.kick_messages[i] = v;
    local = { ...local };
    markDirty();
  }
  function removeKickMsg(i: number) {
    local.auto_reconnect.kick_messages = local.auto_reconnect.kick_messages.filter((_, idx) => idx !== i);
    markDirty();
  }

  const presets: Record<string, { yaw: number; pitch: number }> = {
    North: { yaw: 180, pitch: 0 },
    East: { yaw: -90, pitch: 0 },
    South: { yaw: 0, pitch: 0 },
    West: { yaw: 90, pitch: 0 },
    Up: { yaw: 0, pitch: -90 },
    Down: { yaw: 0, pitch: 90 },
  };
  function setFacing(preset: string) {
    local.facing = { ...presets[preset] };
    markDirty();
  }
  function clearFacing() {
    local.facing = null;
    markDirty();
  }
</script>

<div class="editor">
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <div class="player-icon" aria-hidden="true">
        {#if local.account.kind === 'microsoft'}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
            <circle cx="12" cy="7" r="4"/>
          </svg>
        {/if}
      </div>
      <input
        class="name-input"
        type="text"
        bind:value={local.name}
        on:input={markDirty}
        placeholder="Player name"
        aria-label="Player name"
      />

      <div class="tab-group">
        <button class="tab-btn" class:active={tab === 'config'} on:click={() => (tab = 'config')}>
          Config
        </button>
        <button class="tab-btn" class:active={tab === 'behavior'} on:click={() => { tab = 'behavior'; refreshPreview(); }}>
          Behavior
        </button>
        <button class="tab-btn" class:active={tab === 'console'} on:click={() => (tab = 'console')}>
          Console
        </button>
      </div>
    </div>

    <div class="toolbar-right">
      {#if savedMsg}
        <span class="saved-msg" class:ok={savedOk} class:err={!savedOk}>{savedMsg}</span>
      {/if}
      {#if tab === 'behavior'}
        <button on:click={refreshPreview} class="ghost" title="Refresh preview" aria-label="Refresh preview">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
          </svg>
        </button>
      {/if}
      <button on:click={openWorkdir} class="ghost" title="Open work directory" aria-label="Open work directory">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
      <button class="primary" on:click={save} disabled={saving || !dirty}>
        {#if saving}
          Saving…
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/>
            <polyline points="17 21 17 13 7 13 7 21"/>
            <polyline points="7 3 7 8 15 8"/>
          </svg>
          Save
        {/if}
      </button>
    </div>
  </header>

  <!-- Config Tab -->
  {#if tab === 'config'}
    <div class="form-scroll">
      <!-- Server Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('server')} aria-expanded={sections.server}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"/>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"/>
              <line x1="6" y1="6" x2="6.01" y2="6"/>
              <line x1="6" y1="18" x2="6.01" y2="18"/>
            </svg>
            <span>Server</span>
          </div>
          <span class="chevron" class:open={sections.server} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.server}>
          <div>
            <div class="form-row">
              <div class="form-group" style="flex:3">
                <label for="server-host">Host / IP</label>
                <input id="server-host" type="text" bind:value={local.server.host} on:input={markDirty} placeholder="localhost" />
              </div>
              <div class="form-group" style="flex:1">
                <label for="server-port">Port</label>
                <input id="server-port" type="number" bind:value={local.server.port} on:input={markDirty} placeholder="25565" />
              </div>
            </div>
            <div class="form-group">
              <label for="version-override">Version Override <span class="hint-inline">(blank = auto)</span></label>
              <input id="version-override" type="text" bind:value={local.version_override} on:input={markDirty} placeholder="e.g. 1.21.1" />
            </div>
          </div>
        </div>
      </div>

      <!-- Account Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('account')} aria-expanded={sections.account}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
              <circle cx="12" cy="7" r="4"/>
            </svg>
            <span>Account</span>
          </div>
          <span class="chevron" class:open={sections.account} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.account}>
          <div>
            <div class="account-toggle form-row" style="margin-bottom: var(--space-3)">
              <button class="tab-btn" class:active={local.account.kind === 'offline'} on:click={() => setAccountKind('offline')} style="flex:1">Offline</button>
              <button class="tab-btn" class:active={local.account.kind === 'microsoft'} on:click={() => setAccountKind('microsoft')} style="flex:1">Microsoft</button>
            </div>
            {#if local.account.kind === 'offline'}
              <div class="form-group">
                <label for="acc-username">Username</label>
                <input id="acc-username" type="text" bind:value={local.account.username} on:input={markDirty} placeholder="Steve" />
              </div>
            {:else}
              <div class="form-group">
                <label for="acc-login">Login / Email</label>
                <input id="acc-login" type="text" bind:value={local.account.login} on:input={markDirty} placeholder="player@example.com" />
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- On-Join Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('onjoin')} aria-expanded={sections.onjoin}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <polyline points="4 17 10 11 4 5"/>
              <line x1="12" y1="19" x2="20" y2="19"/>
            </svg>
            <span>On-Join</span>
          </div>
          <span class="chevron" class:open={sections.onjoin} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.onjoin}>
          <div>
            <div class="form-group">
              <label for="join-delay">Delay (seconds)</label>
              <input id="join-delay" type="number" bind:value={local.on_join_delay_secs} on:input={markDirty} min="0" />
            </div>
            <div class="field-label">Commands</div>
            {#each local.on_join_commands as cmd, i}
              <div class="form-row list-row">
                <input type="text" value={cmd} on:input={(e) => updateJoinCmd(i, e.currentTarget.value)} placeholder="/command" />
                <button class="danger icon" on:click={() => removeJoinCmd(i)} style="flex:0 0 auto; width: 34px;" aria-label="Remove command">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                    <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              </div>
            {/each}
            <button on:click={addJoinCmd} style="margin-top: var(--space-1)">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
              </svg>
              Add command
            </button>
          </div>
        </div>
      </div>

      <!-- Facing Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('facing')} aria-expanded={sections.facing}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/>
              <polygon points="16.24 7.76 14.12 14.12 7.76 16.24 9.88 9.88 16.24 7.76"/>
            </svg>
            <span>Facing</span>
          </div>
          <span class="chevron" class:open={sections.facing} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.facing}>
          <div>
            <div class="preset-grid">
              {#each Object.entries(presets) as [label, val]}
                <button on:click={() => setFacing(label)} class:active={local.facing?.yaw === val.yaw && local.facing?.pitch === val.pitch}>
                  {label}
                </button>
              {/each}
            </div>
            <button class="ghost" on:click={clearFacing} style="margin-top: var(--space-1)">Clear facing</button>
            {#if local.facing}
              <div class="form-row" style="margin-top: var(--space-2)">
                <div class="form-group">
                  <label for="facing-yaw">Yaw</label>
                  <input id="facing-yaw" type="number" bind:value={local.facing.yaw} on:input={markDirty} />
                </div>
                <div class="form-group">
                  <label for="facing-pitch">Pitch</label>
                  <input id="facing-pitch" type="number" bind:value={local.facing.pitch} on:input={markDirty} />
                </div>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <!-- Reconnect Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('reconnect')} aria-expanded={sections.reconnect}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
            </svg>
            <span>Auto-Reconnect</span>
          </div>
          <span class="chevron" class:open={sections.reconnect} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.reconnect}>
          <div>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.auto_reconnect.enabled} on:change={markDirty} />
              Enabled
            </label>
            <div class="form-row">
              <div class="form-group">
                <label for="recon-retries">Retries (-1 = infinite)</label>
                <input id="recon-retries" type="number" bind:value={local.auto_reconnect.retries} on:input={markDirty} />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label for="recon-min">Delay min (s)</label>
                <input id="recon-min" type="number" bind:value={local.auto_reconnect.delay_min} on:input={markDirty} />
              </div>
              <div class="form-group">
                <label for="recon-max">Delay max (s)</label>
                <input id="recon-max" type="number" bind:value={local.auto_reconnect.delay_max} on:input={markDirty} />
              </div>
            </div>
            <div class="field-label">Kick Trigger Messages</div>
            {#each local.auto_reconnect.kick_messages as msg, i}
              <div class="form-row list-row">
                <input type="text" value={msg} on:input={(e) => updateKickMsg(i, e.currentTarget.value)} placeholder="Connection has been lost" />
                <button class="danger icon" on:click={() => removeKickMsg(i)} style="flex:0 0 auto; width: 34px;" aria-label="Remove kick message">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" aria-hidden="true">
                    <line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>
                  </svg>
                </button>
              </div>
            {/each}
            <button on:click={addKickMsg} style="margin-top: var(--space-1)">+ Add message</button>
          </div>
        </div>
      </div>

      <!-- Force-Quit Section -->
      <div class="section-card">
        <button class="section-card-header" on:click={() => toggleSection('forcequit')} aria-expanded={sections.forcequit}>
          <div class="section-title-group">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <circle cx="12" cy="12" r="10"/>
              <line x1="15" y1="9" x2="9" y2="15"/>
              <line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
            <span>Force-Quit</span>
          </div>
          <span class="chevron" class:open={sections.forcequit} aria-hidden="true">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9"/>
            </svg>
          </span>
        </button>
        <div class="section-card-body" class:open={sections.forcequit}>
          <div>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.force_quit.on_totem_pop} on:change={markDirty} />
              On totem pop
            </label>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.force_quit.on_attacked} on:change={markDirty} />
              On attacked
            </label>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.force_quit.on_death} on:change={markDirty} />
              On death
            </label>
            <div class="form-group">
              <label for="fq-health">Low health threshold</label>
              <input id="fq-health" type="number" bind:value={local.force_quit.low_health} on:input={markDirty} placeholder="e.g. 5" />
            </div>
            <div class="form-group">
              <label for="fq-regex">Chat / kick regex</label>
              <input id="fq-regex" type="text" bind:value={local.force_quit.chat_regex} on:input={markDirty} placeholder="e.g. You were banned" />
            </div>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.auto_totem} on:change={markDirty} />
              Auto-totem
            </label>
            <label class="switch-label">
              <input type="checkbox" bind:checked={local.eat_when_hungry} on:change={markDirty} />
              Auto-eat when hungry
            </label>
          </div>
        </div>
      </div>
    </div>

  {:else if tab === 'behavior'}
    <!-- Behavior Tab -->
    <div class="behavior-pane">
      <p class="hint">
        Edit the C# behavior script. Config-driven features (facing, on-join, auto-totem, force-quit)
        are merged automatically. Override <code>OnUserJoin()</code>, <code>OnUserUpdate()</code>,
        <code>OnUserChat()</code> to add custom logic.
      </p>
      <div class="editor-grid">
        <div class="pane">
          <div class="pane-label">Your Script → bot.cs</div>
          <div class="pane-body">
            <CodeEditor
              value={local.behaviors_csharp}
              onInput={(v) => { local.behaviors_csharp = v; markDirty(); }}
            />
          </div>
        </div>
        <div class="pane">
          <div class="pane-label">Generated Preview</div>
          <div class="pane-body">
            <CodeEditor value={preview} onInput={() => {}} readonly={true} />
          </div>
        </div>
      </div>
    </div>

  {:else}
    <!-- Console Tab -->
    <Console playerId={local.id} />
  {/if}
</div>

<style>
  .editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* ── Toolbar ── */
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    background: var(--bg-base);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    flex: 1;
    min-width: 0;
  }

  .player-icon {
    width: 36px;
    height: 36px;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .name-input {
    max-width: 200px;
    font-weight: 600;
    font-size: 14px;
    background: transparent;
    border: 1px solid transparent;
    padding: 6px 8px;
    color: var(--text-primary);
  }

  .name-input:hover {
    border-color: var(--border-subtle);
  }

  .name-input:focus {
    background: var(--bg-root);
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-glow);
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .saved-msg {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11.5px;
    font-weight: 600;
  }

  .saved-msg.ok { color: var(--green); }
  .saved-msg.err { color: var(--red); }

  /* ── Config Form ── */
  .form-scroll {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-4) var(--space-4) var(--space-6);
  }

  .section-title-group {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .section-title-group svg {
    color: var(--text-tertiary);
    flex-shrink: 0;
  }

  .chevron {
    color: var(--text-tertiary);
    display: inline-flex;
    transition: transform var(--dur-normal) var(--ease-out-expo);
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    align-items: center;
    justify-content: center;
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .account-toggle {
    display: flex;
    gap: 2px;
    background: var(--bg-hover);
    border-radius: var(--radius-sm);
    padding: 3px;
  }

  .account-toggle .tab-btn {
    border: none;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 500;
    padding: 6px 14px;
  }

  .account-toggle .tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-active);
  }

  .account-toggle .tab-btn.active {
    background: var(--accent-dim);
    color: #ffffff;
    box-shadow: 0 1px 4px var(--accent-glow-strong);
  }

  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 4px;
  }

  .preset-grid button {
    font-size: 11.5px;
    padding: 7px 8px;
  }

  .preset-grid button.active {
    border-color: var(--accent);
    background: var(--accent-glow);
    color: var(--accent);
    font-weight: 600;
  }

  .list-row {
    margin-bottom: 5px;
  }

  /* ── Behavior Tab ── */
  .behavior-pane {
    display: flex;
    flex-direction: column;
    flex: 1;
    padding: var(--space-3) var(--space-4);
    overflow: hidden;
    min-height: 0;
  }

  .editor-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-3);
    flex: 1;
    min-height: 0;
  }

  .pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .pane-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--space-1);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .pane-body {
    flex: 1;
    min-height: 200px;
  }
</style>
