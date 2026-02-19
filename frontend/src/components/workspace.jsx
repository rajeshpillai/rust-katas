import { createSignal, createResource, Show, onCleanup } from "solid-js";
import { fetchKata, runCode } from "../api";
import { currentKataId } from "../state";
import CodePanel from "./code-panel";
import OutputPanel from "./output-panel";

export default function Workspace() {
  const [kata] = createResource(currentKataId, async (id) => {
    if (!id) return null;
    return fetchKata(id);
  });

  const [code, setCode] = createSignal("");
  const [result, setResult] = createSignal(null);
  const [running, setRunning] = createSignal(false);
  const [maximized, setMaximized] = createSignal(null); // null | 'code' | 'output'
  const [splitRatio, setSplitRatio] = createSignal(0.5);
  const [activeView, setActiveView] = createSignal("broken"); // "broken" | "correct" | null
  const [showExplanation, setShowExplanation] = createSignal(false);
  const [showInterpretation, setShowInterpretation] = createSignal(false);

  // Load broken code when kata changes
  let prevKataId = null;
  const loadKata = () => {
    const k = kata();
    if (k && k.id !== prevKataId) {
      prevKataId = k.id;
      setCode(k.broken_code);
      setActiveView("broken");
      setResult(null);
      setShowExplanation(false);
      setShowInterpretation(false);
    }
  };

  // Reactive: runs whenever kata() changes
  const trackKata = () => {
    loadKata();
    return kata();
  };

  async function handleRun() {
    if (running()) return;
    setRunning(true);
    setResult(null);
    try {
      const res = await runCode(code());
      setResult(res);
    } catch (e) {
      setResult({ stdout: "", stderr: "", success: false, execution_time_ms: 0, error: e.message });
    } finally {
      setRunning(false);
    }
  }

  // Resize logic
  let containerRef;
  let isDragging = false;

  function onMouseDown(e) {
    isDragging = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    e.preventDefault();
  }

  function onMouseMove(e) {
    if (!isDragging || !containerRef) return;
    const rect = containerRef.getBoundingClientRect();
    const ratio = (e.clientX - rect.left) / rect.width;
    setSplitRatio(Math.max(0.2, Math.min(0.8, ratio)));
  }

  function onMouseUp() {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    }
  }

  if (typeof document !== "undefined") {
    document.addEventListener("mousemove", onMouseMove);
    document.addEventListener("mouseup", onMouseUp);
    onCleanup(() => {
      document.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseup", onMouseUp);
    });
  }

  function toggleMaximize(panel) {
    setMaximized((cur) => (cur === panel ? null : panel));
  }

  return (
    <div style={{ display: "flex", "flex-direction": "column", flex: "1", "min-width": "0" }}>
      {/* Kata info bar */}
      <Show when={trackKata()}>
        <div style={{
          padding: "0.625rem 1rem",
          "border-bottom": "1px solid var(--border)",
          background: "var(--bg-secondary)",
        }}>
          <div style={{ "font-size": "0.95rem", "font-weight": "600" }}>
            {kata().title}
          </div>
          <div style={{ "font-size": "0.8rem", color: "var(--text-secondary)", "margin-top": "0.25rem" }}>
            {kata().description}
          </div>
        </div>
      </Show>

      {/* Panels */}
      <div class="workspace" ref={containerRef}>
        <CodePanel
          code={code()}
          onCodeChange={(val) => { setCode(val); setActiveView(null); }}
          onRun={handleRun}
          onLoadBroken={() => { setCode(kata()?.broken_code || ""); setActiveView("broken"); setResult(null); }}
          onLoadCorrect={() => { setCode(kata()?.correct_code || ""); setActiveView("correct"); setResult(null); }}
          activeView={activeView()}
          kata={kata()}
          running={running()}
          basis={maximized() === "code" ? "100%" : maximized() === "output" ? "0%" : `${splitRatio() * 100}%`}
          hidden={maximized() === "output"}
          maximized={maximized() === "code"}
          onMaximize={() => toggleMaximize("code")}
        />

        <Show when={!maximized()}>
          <div class="resize-handle" onMouseDown={onMouseDown} />
        </Show>

        <OutputPanel
          result={result()}
          basis={maximized() === "output" ? "100%" : maximized() === "code" ? "0%" : `${(1 - splitRatio()) * 100}%`}
          hidden={maximized() === "code"}
          maximized={maximized() === "output"}
          onMaximize={() => toggleMaximize("output")}
        />
      </div>

      {/* Spoilers */}
      <Show when={kata()}>
        <div style={{ padding: "0 1rem 1rem", background: "var(--bg-primary)" }}>
          <div class="spoiler">
            <button class="spoiler-toggle" onClick={() => setShowExplanation(!showExplanation())}>
              {showExplanation() ? "\u25BC" : "\u25B6"} Explanation
            </button>
            <Show when={showExplanation()}>
              <div class="spoiler-content">{kata().explanation}</div>
            </Show>
          </div>
          <div class="spoiler">
            <button class="spoiler-toggle" onClick={() => setShowInterpretation(!showInterpretation())}>
              {showInterpretation() ? "\u25BC" : "\u25B6"} Compiler Error Interpretation
            </button>
            <Show when={showInterpretation()}>
              <div class="spoiler-content">{kata().compiler_error_interpretation}</div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
