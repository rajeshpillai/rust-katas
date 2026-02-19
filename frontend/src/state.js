import { createSignal } from "solid-js";

const stored = localStorage.getItem("theme");
const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
const initial = stored || (prefersDark ? "dark" : "light");

export const [theme, setTheme] = createSignal(initial);
export const [currentKataId, setCurrentKataId] = createSignal(null);
export const [sidebarExpanded, setSidebarExpanded] = createSignal(true);

export function toggleTheme() {
  const next = theme() === "dark" ? "light" : "dark";
  setTheme(next);
  localStorage.setItem("theme", next);
}

export function initTheme() {
  const t = theme();
  if (t === "dark") {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}
