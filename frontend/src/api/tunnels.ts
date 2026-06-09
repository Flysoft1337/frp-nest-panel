import { api } from './client'
import type { FrpcResponse, TrafficHistoryResponse, Tunnel, TunnelWithTraffic } from './types'

export interface TunnelInput {
  name: string
  protocol: string
  local_host: string
  local_port: number
  remote_port?: number | null
  custom_domain?: string | null
  tls_mode?: string | null
  certificate_id?: string | null
  use_encryption?: boolean
  use_compression?: boolean
  bandwidth_limit?: string | null
  bandwidth_limit_mode?: string | null
  proxy_protocol_version?: string | null
  locations?: string | null
  host_header_rewrite?: string | null
}

export function listTunnels() {
  return api<TunnelWithTraffic[]>('/api/tunnels')
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

export function getTunnelTrafficHistory(id: string, range = '24h') {
  return api<TrafficHistoryResponse>(`/api/tunnels/${id}/traffic-history?range=${encodeURIComponent(range)}`)
}
