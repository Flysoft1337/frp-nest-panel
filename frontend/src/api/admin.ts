import { api } from './client'
import type { ConfigResponse, Invite, Tunnel, UserRow } from './types'

export function getConfig() {
  return api<ConfigResponse>('/api/admin/config')
}

export function listInvites() {
  return api<Invite[]>('/api/admin/invites')
}

export function createInvites(count: number, expiresDays: number | null) {
  return api<Invite[]>('/api/admin/invites', {
    method: 'POST',
    json: { count, expires_days: expiresDays },
  })
}

export function listUsers() {
  return api<UserRow[]>('/api/admin/users')
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

export function listAllTunnels() {
  return api<Tunnel[]>('/api/admin/tunnels')
}

export function deleteTunnel(id: string) {
  return api<{ ok: boolean }>(`/api/admin/tunnels/${id}`, { method: 'DELETE' })
}
