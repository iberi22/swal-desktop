<script>
  let agents = ["Gemini", "Codex", "OpenCode"];
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
</style>
