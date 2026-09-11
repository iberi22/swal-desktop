<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchMeshStatus, getMeshStatusColor, type MeshStatus } from './lib/meshStatus';

  let mesh = $state<MeshStatus | null>(null);
  let meshColor = $state('text-zinc-500');

  onMount(() => {
    void refreshMesh();
    const interval = setInterval(refreshMesh, 30000);
    return () => clearInterval(interval);
  });

  async function refreshMesh() {
    mesh = await fetchMeshStatus();
    meshColor = getMeshStatusColor(mesh.status);
  }
</script>

<main class="min-h-screen bg-zinc-950 text-white p-4">
  <div class="flex items-center gap-4 mb-6">
    <h1 class="text-lg font-medium">SWAL Desktop</h1>
    {#if mesh}
      <span class="flex items-center gap-1.5 text-xs {meshColor}">
        <span class="h-1.5 w-1.5 rounded-full bg-current"></span>
        mesh: {mesh.connected_peers} peers · {mesh.status}
      </span>
    {/if}
  </div>
  <p class="text-sm text-zinc-500">Desktop shell ready.</p>
</main>
