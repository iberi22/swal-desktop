/**
 * Mesh status indicator for SWAL Desktop.
 * Polls Xavier /v1/mesh/health and displays connected peer count.
 */

import { invoke } from '@tauri-apps/api/core';

export interface MeshStatus {
  status: string;
  total_peers: number;
  healthy_peers: number;
  connected_peers: number;
}

/**
 * Fetch mesh health from Xavier.
 * Falls back to degraded status if Xavier is unreachable.
 */
export async function fetchMeshStatus(): Promise<MeshStatus> {
  try {
    const result = await invoke<string>('http_fetch', {
      url: 'http://127.0.0.1:8006/v1/mesh/health',
      method: 'GET',
    });
    return JSON.parse(result) as MeshStatus;
  } catch {
    return {
      status: 'unreachable',
      total_peers: 0,
      healthy_peers: 0,
      connected_peers: 0,
    };
  }
}

/**
 * Get status color for display.
 */
export function getMeshStatusColor(status: string): string {
  switch (status) {
    case 'healthy':
      return 'text-green-400';
    case 'degraded':
      return 'text-yellow-400';
    case 'no_peers':
      return 'text-zinc-500';
    default:
      return 'text-red-400';
  }
}
