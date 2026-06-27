<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { EditorView } from '@codemirror/view';
  import { Compartment } from '@codemirror/state';

  export let value: string;
  export let onInput: (v: string) => void;
  export let readonly: boolean = false;

  let host: HTMLDivElement;
  let view: EditorView | null = null;
  let observer: MutationObserver | null = null;

  function getTheme(): 'dark' | 'light' {
    return document.documentElement.getAttribute('data-theme') === 'light' ? 'light' : 'dark';
  }

  onMount(() => {
    initEditor();
    return () => {
      view?.destroy();
      observer?.disconnect();
    };
  });

  async function initEditor() {
    const [
      { EditorState },
      CMView,
      CMCommands,
      { cpp },
      oneDarkMod,
      { HighlightStyle, syntaxHighlighting },
      { tags },
    ] = await Promise.all([
      import('@codemirror/state'),
      import('@codemirror/view'),
      import('@codemirror/commands'),
      import('@codemirror/lang-cpp'),
      import('@codemirror/theme-one-dark'),
      import('@codemirror/language'),
      import('@lezer/highlight'),
    ]);

    const oneDark = oneDarkMod.oneDark;
    const currentTheme = getTheme();
    const themeComp = new Compartment();

    const updateListener = CMView.EditorView.updateListener.of((u: any) => {
      if (u.docChanged) onInput(u.state.doc.toString());
    });

    function makeLightTheme() {
      const lightEditorTheme = CMView.EditorView.theme({
        '&': {
          backgroundColor: '#f8f9fd',
          color: '#0f172a',
        },
        '.cm-content': {
          caretColor: '#6366f1',
        },
        '.cm-gutters': {
          backgroundColor: '#eef0f8',
          borderRight: '1px solid #e2e5ef',
          color: '#94a3b8',
        },
        '.cm-activeLineGutter': {
          backgroundColor: 'rgba(99, 102, 241, 0.08)',
        },
        '.cm-activeLine': {
          backgroundColor: 'rgba(99, 102, 241, 0.05)',
        },
        '.cm-selectionBackground, .cm-content ::selection': {
          backgroundColor: 'rgba(99, 102, 241, 0.18) !important',
        },
        '.cm-cursor, .cm-dropCursor': {
          borderLeftColor: '#6366f1',
        },
        '.cm-matchingBracket': {
          backgroundColor: 'rgba(99, 102, 241, 0.12)',
          outline: '1px solid rgba(99, 102, 241, 0.4)',
        },
        '.cm-nonmatchingBracket': {
          backgroundColor: 'rgba(239, 68, 68, 0.12)',
        },
        '.cm-foldPlaceholder': {
          backgroundColor: 'transparent',
          border: '1px solid #e2e5ef',
          color: '#94a3b8',
        },
        '.cm-tooltip': {
          border: '1px solid #e2e5ef',
          backgroundColor: '#ffffff',
          color: '#0f172a',
        },
        '.cm-tooltip-autocomplete': {
          '& > ul > li[aria-selected]': {
            backgroundColor: '#eef0f8',
            color: '#0f172a',
          },
        },
      });

      const lightHighlightStyle = HighlightStyle.define([
        { tag: tags.keyword, color: '#7c3aed', fontWeight: '600' },
        { tag: [tags.name, tags.deleted, tags.character, tags.propertyName, tags.macroName], color: '#dc2626' },
        { tag: [tags.function(tags.variableName), tags.labelName], color: '#2563eb' },
        { tag: [tags.color, tags.constant(tags.name), tags.standard(tags.name)], color: '#d97706' },
        { tag: [tags.definition(tags.name), tags.separator], color: '#0f172a' },
        { tag: [tags.typeName, tags.className, tags.namespace, tags.macroName], color: '#2563eb' },
        { tag: [tags.number, tags.bool, tags.null], color: '#d97706' },
        { tag: [tags.string, tags.special(tags.brace)], color: '#059669' },
        { tag: [tags.regexp, tags.escape, tags.special(tags.string)], color: '#db2777' },
        { tag: [tags.comment, tags.lineComment, tags.blockComment], color: '#6b7280', fontStyle: 'italic' },
        { tag: tags.strong, fontWeight: '700' },
        { tag: tags.emphasis, fontStyle: 'italic' },
        { tag: tags.link, color: '#2563eb', textDecoration: 'underline' },
        { tag: [tags.operator, tags.operatorKeyword], color: '#4b5563' },
        { tag: [tags.bracket, tags.paren, tags.squareBracket, tags.brace, tags.angleBracket], color: '#64748b' },
        { tag: [tags.attributeName], color: '#9333ea' },
        { tag: [tags.attributeValue], color: '#059669' },
        { tag: [tags.meta], color: '#dc2626' },
        { tag: [tags.invalid], color: '#dc2626', fontStyle: 'italic' },
        { tag: tags.heading, color: '#0f172a', fontWeight: '700' },
        { tag: tags.quote, color: '#059669', fontStyle: 'italic' },
        { tag: tags.content, color: '#0f172a' },
      ]);

      return [lightEditorTheme, syntaxHighlighting(lightHighlightStyle)];
    }

    view = new CMView.EditorView({
      parent: host,
      state: EditorState.create({
        doc: value,
        extensions: [
          CMCommands.history(),
          CMView.lineNumbers(),
          CMView.keymap.of([...CMCommands.defaultKeymap, ...CMCommands.historyKeymap]),
          cpp(),
          CMView.EditorView.lineWrapping,
          updateListener,
          CMView.EditorView.editable.of(!readonly),
          themeComp.of(currentTheme === 'dark' ? oneDark : makeLightTheme()),
        ],
      }),
    });

    observer = new MutationObserver(() => {
      const newTheme = getTheme();
      if (view) {
        view.dispatch({
          effects: themeComp.reconfigure(newTheme === 'dark' ? oneDark : makeLightTheme()),
        });
      }
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
  }

  $: if (view && value !== view.state.doc.toString()) {
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: value },
    });
  }

  onDestroy(() => {
    view?.destroy();
    observer?.disconnect();
  });
</script>

<div class="cm-host" bind:this={host}></div>

<style>
  .cm-host {
    height: 100%;
    overflow: hidden;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .cm-host :global(.cm-editor) {
    height: 100%;
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .cm-host :global(.cm-scroller) {
    overflow: auto;
    font-family: 'Fira Code', 'JetBrains Mono', 'SFMono-Regular', Consolas, monospace;
    font-size: 12.5px;
  }
</style>
