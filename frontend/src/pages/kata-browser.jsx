import ThemeToggle from "../components/theme-toggle";
import Sidebar from "../components/sidebar";
import Workspace from "../components/workspace";

export default function KataBrowser() {
  return (
    <div style={{ display: "flex", height: "100vh", overflow: "hidden" }}>
      <Sidebar />
      <div style={{ display: "flex", "flex-direction": "column", flex: "1", "min-width": "0" }}>
        <div style={{
          display: "flex",
          "align-items": "center",
          "justify-content": "space-between",
          padding: "0.5rem 1rem",
          "border-bottom": "1px solid var(--border)",
          background: "var(--bg-secondary)",
          "flex-shrink": "0",
        }}>
          <a href="#/" style={{
            "text-decoration": "none",
            color: "var(--text-primary)",
            "font-weight": "600",
            "font-size": "0.9rem",
          }}>
            Rust Katas
          </a>
          <ThemeToggle />
        </div>
        <Workspace />
      </div>
    </div>
  );
}
