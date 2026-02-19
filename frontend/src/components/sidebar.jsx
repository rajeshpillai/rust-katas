import { createSignal, createResource, createEffect, For, Show } from "solid-js";
import { fetchKatas } from "../api";
import { sidebarExpanded, setSidebarExpanded, currentKataId, setCurrentKataId } from "../state";

export default function Sidebar(props) {
  const [kataList] = createResource(fetchKatas);
  const [openPhases, setOpenPhases] = createSignal(new Set([0]));

  // Auto-expand the phase containing the current kata (e.g. from URL)
  createEffect(() => {
    const list = kataList();
    const id = currentKataId();
    if (!list || !id) return;
    for (const phase of list.phases) {
      if (phase.katas.some((k) => k.id === id)) {
        setOpenPhases((prev) => {
          if (prev.has(phase.phase)) return prev;
          const next = new Set(prev);
          next.add(phase.phase);
          return next;
        });
        break;
      }
    }
  });

  function togglePhase(phase) {
    setOpenPhases((prev) => {
      const next = new Set(prev);
      if (next.has(phase)) next.delete(phase);
      else next.add(phase);
      return next;
    });
  }

  function selectKata(id) {
    setCurrentKataId(id);
    window.location.hash = `#/katas/${id}`;
    if (props.onSelectKata) props.onSelectKata(id);
  }

  return (
    <aside class={`sidebar ${sidebarExpanded() ? "" : "sidebar--collapsed"}`}>
      <div class="sidebar-header">
        <button
          class="burger-btn"
          onClick={() => setSidebarExpanded(!sidebarExpanded())}
          title={sidebarExpanded() ? "Collapse sidebar" : "Expand sidebar"}
        >
          <svg width="18" height="18" viewBox="0 0 18 18" fill="none">
            <path d="M2 4h14M2 9h14M2 14h14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
        <Show when={sidebarExpanded()}>
          <span style={{ "font-size": "0.85rem", "font-weight": "600" }}>Phases</span>
        </Show>
      </div>

      <Show when={sidebarExpanded()}>
        <div style={{ flex: "1", "overflow-y": "auto" }}>
          <Show when={kataList()} fallback={<div style={{ padding: "1rem", color: "var(--text-muted)", "font-size": "0.8rem" }}>Loading...</div>}>
            <For each={kataList().phases}>
              {(phase) => (
                <div class="phase-group">
                  <div class="phase-header" onClick={() => togglePhase(phase.phase)}>
                    <span class={`phase-chevron ${openPhases().has(phase.phase) ? "phase-chevron--open" : ""}`}>
                      &#9656;
                    </span>
                    <span>Phase {phase.phase} &mdash; {phase.title}</span>
                  </div>
                  <Show when={openPhases().has(phase.phase)}>
                    <For each={phase.katas}>
                      {(kata) => (
                        <div
                          class={`kata-link ${currentKataId() === kata.id ? "kata-link--active" : ""}`}
                          onClick={() => selectKata(kata.id)}
                        >
                          {kata.sequence}. {kata.title}
                        </div>
                      )}
                    </For>
                  </Show>
                </div>
              )}
            </For>
          </Show>
        </div>
      </Show>
    </aside>
  );
}
