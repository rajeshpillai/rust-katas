const BASE = "/api";

export async function fetchKatas() {
  const res = await fetch(`${BASE}/katas`);
  if (!res.ok) throw new Error(`Failed to fetch katas: ${res.status}`);
  return res.json();
}

export async function fetchKata(id) {
  const res = await fetch(`${BASE}/katas/${id}`);
  if (!res.ok) throw new Error(`Failed to fetch kata: ${res.status}`);
  return res.json();
}

export async function runCode(code) {
  const res = await fetch(`${BASE}/playground/run`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ code }),
  });
  if (!res.ok) throw new Error(`Failed to run code: ${res.status}`);
  return res.json();
}
