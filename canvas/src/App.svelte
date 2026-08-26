<script>
  import { onMount } from "svelte";
  let agents = ["Gemini", "Codex", "OpenCode"];

  // Daemons locales — auto-discovery (OLA-S3.15)
  let daemons = [
    { service: "xavier-api", url: "http://127.0.0.1:8006/health", latency_ms: null, ok: null },
    { service: "oauth-proxy", url: "http://127.0.0.1:8200/health", latency_ms: null, ok: null },
    { service: "xavier-mcp", url: "http://127.0.0.1:8100", latency_ms: null, ok: null },
  ];
  let detecting = false;

  async function detectar() {
    detecting = true;
    try {
      // Try Tauri invoke if available, else fallback to direct fetch simulation
      if (typeof window !== "undefined" && window.__TAURI__?.core?.invoke) {
        const res = await window.__TAURI__.core.invoke("discover_local_daemons");
        if (Array.isArray(res) && res.length === 3) {
          daemons = res;
          persistEndpoints();
        }
      } else {
        // Fallback: sequential fetch with 800ms timeout each (mirrors Rust logic)
        for (let d of daemons) {
          const start = Date.now();
          try {
            if (d.service === "xavier-mcp") {
              // TCP check not possible from browser; mark as unknown
              d.latency_ms = Date.now() - start;
              d.ok = false;
            } else {
              const ctrl = new AbortController();
              const t = setTimeout(() => ctrl.abort(), 800);
              const r = await fetch(d.url, { signal: ctrl.signal });
              clearTimeout(t);
              d.latency_ms = Date.now() - start;
              d.ok = r.ok;
            }
          } catch (e) {
            d.latency_ms = Date.now() - start;
            d.ok = false;
          }
        }
        daemons = [...daemons];
        persistEndpoints();
      }
    } finally {
      detecting = false;
    }
  }

  function persistEndpoints() {
    try {
      const chosen = daemons.filter(d => d.ok).reduce((acc, d) => ({ ...acc, [d.service]: d.url }), {});
      localStorage.setItem("swal:daemon_endpoints", JSON.stringify(chosen));
    } catch {}
  }

  function statusDot(ok) {
    if (ok === null) return "●";
    return ok ? "🟢" : "🔴";
  }
  function statusColor(ok) {
    if (ok === null) return "#f59e0b";
    return ok ? "#10b981" : "#ef4444";
  }
</script>

<main>
  <h1>⚡ SWAL AI Canvas</h1>
  <div class="canvas">
    <p>Lienzo interactivo para agentes IA</p>
    <div class="agent-grid">
      {#each agents as agent}
        <div class="agent-card">
          <h3>{agent}</h3>
          <p>Status: Ready</p>
        </div>
      {/each}
    </div>

    <!-- Daemons locales — OLA-S3.15 -->
    <section class="daemons-locales">
      <h2>Daemons locales</h2>
      <button on:click={detectar} disabled={detecting}>{detecting ? "Detectando..." : "Detectar"}</button>
      <table>
        <thead>
          <tr><th>Servicio</th><th>URL</th><th>Estado</th><th>Latencia</th></tr>
        </thead>
        <tbody>
          {#each daemons as d}
            <tr>
              <td>{d.service}</td>
              <td><code>{d.url}</code></td>
              <td><span class="dot" style:color={statusColor(d.ok)}>{statusDot(d.ok)}</span> {d.ok === null ? "pendiente" : d.ok ? "ok" : "offline"}</td>
              <td>{d.latency_ms === null ? "--" : d.latency_ms + " ms"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="hint">Endpoints elegidos se guardan en app config (localStorage + Rust settings_store)</p>
    </section>
  </div>
</main>

<style>
  main {
    text-align: center;
    padding: 1em;
    background: #0D1117;
    color: #00FF88;
    height: 100vh;
  }
  .agent-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 2rem;
  }
  .agent-card {
    border: 1px solid #7DCFFF;
    padding: 1rem;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
  }
  .daemons-locales {
    margin-top: 2rem;
    padding: 1.5rem;
    border: 1px solid #30363d;
    border-radius: 8px;
    background: #161b22;
    text-align: left;
  }
  .daemons-locales h2 { color: #7DCFFF; margin-bottom: 0.75rem; }
  .daemons-locales button {
    background: #0078D4;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    margin-bottom: 1rem;
  }
  .daemons-locales button:disabled { opacity: 0.5; cursor: wait; }
  .daemons-locales table { width: 100%; border-collapse: collapse; }
  .daemons-locales th, .daemons-locales td { padding: 0.5rem; border-bottom: 1px solid #30363d; font-size: 0.9rem; }
  .daemons-locales th { color: #8b949e; text-align: left; }
  .daemons-locales .dot { font-size: 1.2rem; }
  .daemons-locales .hint { color: #8b949e; font-size: 0.8rem; margin-top: 0.75rem; }
</style>
