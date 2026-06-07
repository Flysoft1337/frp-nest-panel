import { api } from './client'
import type { SessionResponse } from './types'

export function getSession() {
  return api<SessionResponse>('/api/session')
}

export function login(username: string, password: string) {
  return api<SessionResponse>('/api/login', {
    method: 'POST',
    json: { username, password },
  })
}

export function register(inviteCode: string, username: string, password: string) {
  return api<SessionResponse>('/api/register', {
    method: 'POST',
    json: { invite_code: inviteCode, username, password },
  })
}

export function logout() {
  return api<{ ok: boolean }>('/api/logout', { method: 'POST' })
}

export function changePassword(currentPassword: string, newPassword: string, confirmPassword: string) {
  return api<{ ok: boolean }>('/api/password', {
    method: 'POST',
    json: {
      current_password: currentPassword,
      new_password: newPassword,
      confirm_password: confirmPassword,
    },
  })
}
