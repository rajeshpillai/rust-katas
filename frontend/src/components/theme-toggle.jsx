import { createEffect } from "solid-js";
import { theme, toggleTheme } from "../state";

export default function ThemeToggle() {
  createEffect(() => {
    if (theme() === "dark") {
      document.documentElement.classList.add("dark");
    } else {
      document.documentElement.classList.remove("dark");
    }
  });

  return (
    <button class="theme-toggle" onClick={toggleTheme} title="Toggle theme">
      {theme() === "dark" ? "\u2600" : "\u263E"}
    </button>
  );
}
