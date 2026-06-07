import { api } from './client'
import type { FrpcResponse, Tunnel } from './types'

export interface TunnelInput {
  name: string
  protocol: string
  local_host: string
  local_port: number
}

export function listTunnels() {
  return api<Tunnel[]>('/api/tunnels')
}

export function createTunnel(input: TunnelInput) {
  return api<Tunnel>('/api/tunnels', {
    method: 'POST',
    json: input,
  })
}

export function getTunnel(id: string) {
  return api<Tunnel>(`/api/tunnels/${id}`)
}

export function updateTunnel(id: string, input: TunnelInput) {
  return api<Tunnel>(`/api/tunnels/${id}`, {
    method: 'PATCH',
    json: input,
  })
}

export function deleteTunnel(id: string) {
  return api<{ ok: boolean }>(`/api/tunnels/${id}`, { method: 'DELETE' })
}

export function getFrpc(id: string) {
  return api<FrpcResponse>(`/api/tunnels/${id}/frpc`)
}
