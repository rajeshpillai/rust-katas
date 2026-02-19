import { Show } from "solid-js";

export default function CodePanel(props) {
  function handleKeyDown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      if (props.onRun) props.onRun();
    }
    // Allow Tab to insert spaces in textarea
    if (e.key === "Tab") {
      e.preventDefault();
      const ta = e.target;
      const start = ta.selectionStart;
      const end = ta.selectionEnd;
      const value = ta.value;
      const newValue = value.substring(0, start) + "    " + value.substring(end);
      ta.value = newValue;
      ta.selectionStart = ta.selectionEnd = start + 4;
      if (props.onCodeChange) props.onCodeChange(newValue);
    }
  }

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
      <textarea
        class="code-textarea"
        value={props.code || ""}
        onInput={(e) => props.onCodeChange?.(e.target.value)}
        onKeyDown={handleKeyDown}
        spellcheck={false}
        placeholder="// Write your Rust code here..."
      />
    </div>
  );
}
