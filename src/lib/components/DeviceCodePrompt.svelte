<script lang="ts">
  import { open } from '@tauri-apps/plugin-shell';

  export let prompt: { code: string; url: string } | null;
  export let playerName: string;

  let copied = false;
  let closing = false;

  async function copy() {
    if (!prompt) return;
    try {
      await navigator.clipboard.writeText(prompt.code);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      window.prompt('Copy this code:', prompt.code);
    }
  }

  async function openUrl() {
    if (!prompt) return;
    try {
      await open(prompt.url);
    } catch {
      window.open(prompt.url, '_blank');
    }
  }

  function dismiss() {
    closing = true;
    setTimeout(() => {
      prompt = null;
      closing = false;
    }, 250);
  }
</script>

{#if prompt}
  <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
  <div
    class="overlay animate-fade"
    class:closing
    role="dialog"
    aria-modal="true"
    on:keydown={(e) => e.key === 'Escape' && dismiss()}
  >
    <div class="modal animate-scale">
      <!-- Header -->
      <div class="modal-header">
        <div class="modal-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
        </div>
        <div>
          <h3>Microsoft Sign-In</h3>
          <p>Player <strong>{playerName}</strong> needs to authenticate</p>
        </div>
      </div>

      <!-- Steps -->
      <div class="steps">
        <!-- Step 1 -->
        <div class="step">
          <span class="step-num">1</span>
          <div class="step-content">
            <div class="step-title">Open the Microsoft login page</div>
            <a href={prompt.url} on:click|preventDefault={openUrl} class="step-link">{prompt.url}</a>
            <button class="primary" on:click={openUrl} style="margin-top: 6px;">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
                <polyline points="15 3 21 3 21 9"/>
                <line x1="10" y1="14" x2="21" y2="3"/>
              </svg>
              Open in Browser
            </button>
          </div>
        </div>

        <!-- Separator -->
        <div class="step-sep">
          <span>then</span>
        </div>

        <!-- Step 2 -->
        <div class="step">
          <span class="step-num">2</span>
          <div class="step-content">
            <div class="step-title">Enter this code</div>
            <div class="code-display">
              <code>{prompt.code}</code>
            </div>
            <button on:click={copy}>
              {#if copied}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
                Copied!
              {:else}
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
                </svg>
                Copy Code
              {/if}
            </button>
          </div>
        </div>

        <!-- Separator -->
        <div class="step-sep">
          <span>then</span>
        </div>

        <!-- Step 3 -->
        <div class="step">
          <span class="step-num">3</span>
          <div class="step-content">
            <div class="step-title">Complete sign-in in your browser</div>
            <p class="step-hint">MCC will automatically pick up the token after you finish.</p>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <button class="ghost" on:click={dismiss}>Dismiss</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--overlay-bg);
    backdrop-filter: var(--glass-blur);
    -webkit-backdrop-filter: var(--glass-blur);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .overlay.closing {
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--dur-normal) var(--ease-in-out);
  }

  .modal {
    background: var(--bg-overlay);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-xl);
    padding: var(--space-6);
    max-width: 480px;
    width: 92%;
    box-shadow: var(--shadow-lg), var(--shadow-glow-green);
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    margin-bottom: var(--space-5);
  }

  .modal-icon {
    width: 46px;
    height: 46px;
    border-radius: var(--radius-md);
    background: var(--accent-glow);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .modal h3 {
    font-size: 17px;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
    text-transform: none;
  }

  .modal-header p {
    margin-top: 2px;
    font-size: 13px;
    color: var(--text-secondary);
  }

  .modal-header :global(strong) {
    color: var(--accent);
    font-weight: 600;
  }

  .steps {
    display: flex;
    flex-direction: column;
  }

  .step {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-2) 0;
  }

  .step-num {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: var(--accent-glow);
    color: var(--accent);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    font-weight: 700;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .step-content {
    flex: 1;
    min-width: 0;
  }

  .step-title {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: 6px;
  }

  .step-link {
    font-size: 11.5px;
    color: var(--accent);
    word-break: break-all;
    display: block;
    margin-bottom: 4px;
    text-decoration: none;
  }

  .step-link:hover {
    text-decoration: underline;
  }

  .step-hint {
    font-size: 12px;
    color: var(--text-tertiary);
  }

  .code-display {
    background: var(--bg-root);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-md);
    padding: var(--space-3) var(--space-4);
    margin-bottom: var(--space-2);
    text-align: center;
  }

  .code-display code {
    font-size: 26px;
    font-weight: 700;
    letter-spacing: 0.18em;
    color: var(--accent);
    font-family: var(--font-mono);
    background: transparent;
    padding: 0;
  }

  .step-sep {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding-left: 44px;
  }

  .step-sep::before {
    content: '';
    width: 2px;
    height: 18px;
    background: var(--border-subtle);
    border-radius: 1px;
  }

  .step-sep span {
    font-size: 10.5px;
    color: var(--text-tertiary);
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.06em;
  }

  .modal-footer {
    margin-top: var(--space-4);
    padding-top: var(--space-3);
    border-top: 1px solid var(--border-subtle);
    display: flex;
    justify-content: flex-end;
  }
</style>
