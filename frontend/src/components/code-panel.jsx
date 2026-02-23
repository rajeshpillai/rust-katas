import { Show, onMount, onCleanup, createEffect } from "solid-js";
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightSpecialChars } from "@codemirror/view";
import { EditorState, Compartment } from "@codemirror/state";
import { defaultKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
import { rust } from "@codemirror/lang-rust";
import { theme as appTheme } from "../state.js";

const lightTheme = EditorView.theme({
  "&": { backgroundColor: "var(--bg-code)", color: "var(--text-primary)" },
  ".cm-content": { caretColor: "var(--text-primary)", fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", "Consolas", monospace', fontSize: "0.875rem", lineHeight: "1.6" },
  ".cm-gutters": { backgroundColor: "var(--bg-secondary)", color: "var(--text-muted)", border: "none" },
  ".cm-activeLineGutter": { backgroundColor: "var(--bg-tertiary)" },
  ".cm-activeLine": { backgroundColor: "var(--bg-tertiary)" },
  ".cm-selectionBackground": { backgroundColor: "rgba(59, 130, 246, 0.2) !important" },
  "&.cm-focused .cm-selectionBackground": { backgroundColor: "rgba(59, 130, 246, 0.3) !important" },
  ".cm-cursor": { borderLeftColor: "var(--text-primary)" },
  ".cm-matchingBracket": { backgroundColor: "rgba(59, 130, 246, 0.25)", outline: "1px solid rgba(59, 130, 246, 0.5)" },
});

const darkTheme = EditorView.theme({
  "&": { backgroundColor: "var(--bg-code)", color: "var(--text-primary)" },
  ".cm-content": { caretColor: "var(--text-primary)", fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", "Consolas", monospace', fontSize: "0.875rem", lineHeight: "1.6" },
  ".cm-gutters": { backgroundColor: "var(--bg-secondary)", color: "var(--text-muted)", border: "none" },
  ".cm-activeLineGutter": { backgroundColor: "var(--bg-tertiary)" },
  ".cm-activeLine": { backgroundColor: "var(--bg-tertiary)" },
  ".cm-selectionBackground": { backgroundColor: "rgba(96, 165, 250, 0.2) !important" },
  "&.cm-focused .cm-selectionBackground": { backgroundColor: "rgba(96, 165, 250, 0.3) !important" },
  ".cm-cursor": { borderLeftColor: "var(--text-primary)" },
  ".cm-matchingBracket": { backgroundColor: "rgba(96, 165, 250, 0.25)", outline: "1px solid rgba(96, 165, 250, 0.5)" },
}, { dark: true });

const darkHighlight = syntaxHighlighting(defaultHighlightStyle, { fallback: true });

export default function CodePanel(props) {
  let containerRef;
  let view;
  const themeCompartment = new Compartment();
  // Flag to prevent feedback loop when we set code externally
  let settingExternally = false;

  function getThemeExtension() {
    return appTheme() === "dark" ? darkTheme : lightTheme;
  }

  onMount(() => {
    const runKeymap = keymap.of([{
      key: "Mod-Enter",
      run: () => { if (props.onRun) props.onRun(); return true; },
    }]);

    const updateListener = EditorView.updateListener.of((update) => {
      if (update.docChanged && !settingExternally) {
        props.onCodeChange?.(update.state.doc.toString());
      }
    });

    view = new EditorView({
      state: EditorState.create({
        doc: props.code || "",
        extensions: [
          lineNumbers(),
          highlightActiveLine(),
          highlightSpecialChars(),
          bracketMatching(),
          darkHighlight,
          rust(),
          keymap.of([...defaultKeymap, indentWithTab]),
          runKeymap,
          updateListener,
          themeCompartment.of(getThemeExtension()),
          EditorView.lineWrapping,
        ],
      }),
      parent: containerRef,
    });
  });

  onCleanup(() => {
    if (view) view.destroy();
  });

  // Sync external code changes (Broken/Correct buttons)
  createEffect(() => {
    const newCode = props.code ?? "";
    if (view && view.state.doc.toString() !== newCode) {
      settingExternally = true;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: newCode },
      });
      settingExternally = false;
    }
  });

  // Sync theme changes
  createEffect(() => {
    const _ = appTheme(); // subscribe to signal
    if (view) {
      view.dispatch({
        effects: themeCompartment.reconfigure(getThemeExtension()),
      });
    }
  });

  return (
    <div class="panel" style={{ "flex-basis": props.basis || "50%", display: props.hidden ? "none" : "flex" }}>
      <div class="panel-header">
        <span>Code</span>
        <div style={{ display: "flex", gap: "0.375rem", "align-items": "center" }}>
          <Show when={props.kata}>
            <button
              class={`btn btn--small ${props.activeView === "broken" ? "btn--active" : ""}`}
              onClick={() => props.onLoadBroken?.()}
            >
              Broken
            </button>
            <button
              class={`btn btn--small ${props.activeView === "correct" ? "btn--active" : ""}`}
              onClick={() => props.onLoadCorrect?.()}
            >
              Correct
            </button>
          </Show>
          <button
            class="btn btn--small btn--primary"
            onClick={() => props.onRun?.()}
            disabled={props.running}
          >
            {props.running ? "Running..." : "Run"}
          </button>
          <button class="maximize-btn" onClick={() => props.onMaximize?.()} title="Maximize">
            {props.maximized ? "\u25A3" : "\u25A1"}
          </button>
        </div>
      </div>
      <div ref={containerRef} class="code-editor-container" />
    </div>
  );
}
