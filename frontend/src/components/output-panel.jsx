import { Show } from "solid-js";

export default function OutputPanel(props) {
  return (
    <div class="panel" style={{ "flex-basis": props.basis || "50%", display: props.hidden ? "none" : "flex" }}>
      <div class="panel-header">
        <span>Output</span>
        <button class="maximize-btn" onClick={() => props.onMaximize?.()} title="Maximize">
          {props.maximized ? "\u25A3" : "\u25A1"}
        </button>
      </div>
      <div class="output-area">
        <Show when={props.result} fallback={
          <span style={{ color: "var(--text-muted)" }}>Click "Run" or press Ctrl+Enter to execute your code.</span>
        }>
          <Show when={props.result.error}>
            <div class="output-error">{props.result.error}</div>
          </Show>
          <Show when={props.result.stderr}>
            <div class="output-stderr">{props.result.stderr}</div>
          </Show>
          <Show when={props.result.stdout}>
            <div class="output-stdout">{props.result.stdout}</div>
          </Show>
          <div class="output-meta">
            {props.result.success ? "Exited successfully" : "Exited with error"}
            {" \u2022 "}
            {props.result.execution_time_ms}ms
          </div>
        </Show>
      </div>
    </div>
  );
}
