<script lang="ts">
  import { onMount } from 'svelte';
  import { fetchMeshStatus, type MeshStatus } from '../lib/meshStatus';

  let mesh = $state<MeshStatus | null>(null);
  let expanded = $state(false);

  onMount(() => {
    void refresh();
    const interval = setInterval(refresh, 30000);
    return () => clearInterval(interval);
  });

  async function refresh() {
    mesh = await fetchMeshStatus();
  }
</script>

<button 
  class="mesh-minimap" 
  onclick={() => expanded = !expanded}
  title="Mesh Status"
>
  {#if mesh}
    <span class="h-2 w-2 rounded-full {mesh.status === 'healthy' ? 'bg-green-400' : mesh.status === 'degraded' ? 'bg-yellow-400' : 'bg-zinc-500'}"></span>
    <span class="text-[10px] text-zinc-400">{mesh.connected_peers}</span>
  {:else}
    <span class="h-2 w-2 rounded-full bg-zinc-600"></span>
  {/if}
</button>

{#if expanded && mesh}
  <div class="mesh-popup">
    <p class="text-xs text-zinc-300">Status: {mesh.status}</p>
    <p class="text-xs text-zinc-500">Peers: {mesh.connected_peers}/{mesh.total_peers}</p>
    <p class="text-xs text-zinc-500">Healthy: {mesh.healthy_peers}</p>
  </div>
{/if}

<style>
  .mesh-minimap {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    cursor: pointer;
  }
  .mesh-minimap:hover {
    background: rgba(255, 255, 255, 0.1);
  }
  .mesh-popup {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    padding: 8px;
    border-radius: 8px;
    background: #18181b;
    border: 1px solid rgba(255, 255, 255, 0.1);
    z-index: 50;
    min-width: 120px;
  }
</style>
