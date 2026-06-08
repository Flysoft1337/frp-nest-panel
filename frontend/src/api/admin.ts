import { api } from './client'
import type {
  AdminSummary,
  AdminTrafficSummary,
  AdminTunnelRow,
  ConfigResponse,
  FrpsStatus,
  FrpsUpdate,
  PanelTlsStatus,
  Invite,
  ListParams,
  PageResponse,
  UserRow,
} from './types'

function withParams(path: string, params: ListParams = {}) {
  const query = new URLSearchParams()
  if (params.q) query.set('q', params.q)
  if (params.status) query.set('status', params.status)
  if (params.page) query.set('page', String(params.page))
  if (params.page_size) query.set('page_size', String(params.page_size))
  const text = query.toString()
  return text ? `${path}?${text}` : path
}

export function getConfig() {
  return api<ConfigResponse>('/api/admin/config')
}

export function getAdminSummary() {
  return api<AdminSummary>('/api/admin/summary')
}

export function getAdminTrafficSummary() {
  return api<AdminTrafficSummary>('/api/admin/traffic/summary')
}

export function listInvites(params?: ListParams) {
  return api<PageResponse<Invite>>(withParams('/api/admin/invites', params))
}

export function createInvites(count: number, expiresDays: number | null) {
  return api<Invite[]>('/api/admin/invites', {
    method: 'POST',
    json: { count, expires_days: expiresDays },
  })
}

export function deleteInvite(id: string) {
  return api<{ ok: boolean }>(`/api/admin/invites/${id}`, { method: 'DELETE' })
}

export function listUsers(params?: ListParams) {
  return api<PageResponse<UserRow>>(withParams('/api/admin/users', params))
}

export function enableUser(id: string) {
  return api<{ ok: boolean }>(`/api/admin/users/${id}/enable`, { method: 'POST' })
}

export function disableUser(id: string) {
  return api<{ ok: boolean }>(`/api/admin/users/${id}/disable`, { method: 'POST' })
}

export function resetUserPassword(id: string, newPassword: string) {
  return api<{ ok: boolean }>(`/api/admin/users/${id}/reset-password`, {
    method: 'POST',
    json: { new_password: newPassword },
  })
}

export function updateUserQuota(id: string, maxTunnels: number | null) {
  return api<{ ok: boolean }>(`/api/admin/users/${id}/quota`, {
    method: 'POST',
    json: { max_tunnels: maxTunnels },
  })
}

export function listAllTunnels(params?: ListParams) {
  return api<PageResponse<AdminTunnelRow>>(withParams('/api/admin/tunnels', params))
}

export function deleteTunnel(id: string) {
  return api<{ ok: boolean }>(`/api/admin/tunnels/${id}`, { method: 'DELETE' })
}

export function getFrps() {
  return api<FrpsStatus>('/api/admin/frps')
}

export function updateFrps(form: FrpsUpdate) {
  return api<{ ok: boolean }>('/api/admin/frps', { method: 'PUT', json: form })
}

export function restartFrps() {
  return api<{ ok: boolean }>('/api/admin/frps/restart', { method: 'POST' })
}

export function getPanelTls() {
  return api<PanelTlsStatus>('/api/admin/panel-tls')
}

export function updatePanelTls(form: {
  enabled: boolean
  bind: string
  certificate_pem?: string | null
  private_key_pem?: string | null
}) {
  return api<PanelTlsStatus>('/api/admin/panel-tls', { method: 'PUT', json: form })
}
