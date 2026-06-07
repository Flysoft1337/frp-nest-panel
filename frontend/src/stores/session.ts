import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import * as authApi from '../api/auth'
import { ApiError } from '../api/client'
import type { PublicUser } from '../api/types'

export const useSessionStore = defineStore('session', () => {
  const user = ref<PublicUser | null>(null)
  const loaded = ref(false)

  const isAuthenticated = computed(() => Boolean(user.value))
  const isAdmin = computed(() => user.value?.role === 'admin')

  async function load() {
    try {
      const session = await authApi.getSession()
      user.value = session.user
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) {
        user.value = null
      } else {
        throw error
      }
    } finally {
      loaded.value = true
    }
  }

  async function login(username: string, password: string) {
    const session = await authApi.login(username, password)
    user.value = session.user
    loaded.value = true
  }

  async function register(inviteCode: string, username: string, password: string) {
    const session = await authApi.register(inviteCode, username, password)
    user.value = session.user
    loaded.value = true
  }

  async function logout() {
    await authApi.logout()
    user.value = null
    loaded.value = true
  }

  return { user, loaded, isAuthenticated, isAdmin, load, login, register, logout }
})
