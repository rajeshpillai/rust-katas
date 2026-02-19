import ThemeToggle from "../components/theme-toggle";

export default function Landing() {
  return (
    <div style={{
      "min-height": "100vh",
      display: "flex",
      "flex-direction": "column",
      "align-items": "center",
      "justify-content": "center",
      padding: "2rem",
    }}>
      <div style={{
        position: "absolute",
        top: "1rem",
        right: "1rem",
      }}>
        <ThemeToggle />
      </div>

      <h1 style={{
        "font-size": "2.5rem",
        "font-weight": "700",
        "margin-bottom": "0.5rem",
      }}>
        Rust Katas
      </h1>
      <p style={{
        color: "var(--text-secondary)",
        "margin-bottom": "3rem",
        "text-align": "center",
        "max-width": "480px",
        "font-size": "1rem",
      }}>
        A discipline of correctness. Learn Rust through deliberate practice.
      </p>

      <div style={{
        display: "grid",
        "grid-template-columns": "repeat(auto-fit, minmax(260px, 1fr))",
        gap: "1.5rem",
        "max-width": "600px",
        width: "100%",
      }}>
        <a href="#/katas" style={{ "text-decoration": "none", color: "inherit" }}>
          <div class="card" style={{ cursor: "pointer" }}>
            <div style={{ "font-size": "2rem", "margin-bottom": "0.75rem" }}>&#x1F9E0;</div>
            <h2 style={{ "font-size": "1.25rem", "font-weight": "600", "margin": "0 0 0.5rem" }}>
              Katas
            </h2>
            <p style={{ color: "var(--text-secondary)", "font-size": "0.875rem", margin: "0" }}>
              Structured learning sequence. Build intuition through broken code, compiler errors, and invariants.
            </p>
          </div>
        </a>

        <div class="card card--disabled">
          <div style={{ "font-size": "2rem", "margin-bottom": "0.75rem" }}>&#x1F680;</div>
          <h2 style={{ "font-size": "1.25rem", "font-weight": "600", margin: "0 0 0.5rem" }}>
            Applications
          </h2>
          <p style={{ color: "var(--text-secondary)", "font-size": "0.875rem", margin: "0" }}>
            Real-world Rust + WASM projects. Coming after kata completion.
          </p>
          <span style={{
            display: "inline-block",
            "margin-top": "0.75rem",
            padding: "0.25rem 0.625rem",
            "border-radius": "1rem",
            "font-size": "0.7rem",
            "font-weight": "600",
            background: "var(--bg-tertiary)",
            color: "var(--text-muted)",
          }}>
            Coming Soon
          </span>
        </div>
      </div>
    </div>
  );
}
