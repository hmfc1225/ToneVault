import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User } from '@/types'
import * as authApi from '@/api/auth'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const accessToken = ref(localStorage.getItem('access_token') || '')
  const refreshToken = ref(localStorage.getItem('refresh_token') || '')

  const isAuthenticated = computed(() => !!accessToken.value)

  async function setup(username: string, password: string, displayName?: string, email?: string) {
    const res = await authApi.setup(username, password, displayName, email)
    setTokens(res.access_token, res.refresh_token)
    user.value = res.user
  }

  async function login(username: string, password: string) {
    const res = await authApi.login(username, password)
    setTokens(res.access_token, res.refresh_token)
    user.value = res.user
  }

  async function fetchMe() {
    try {
      user.value = await authApi.getMe()
    } catch {
      logout()
    }
  }

  function logout() {
    user.value = null
    accessToken.value = ''
    refreshToken.value = ''
    localStorage.removeItem('access_token')
    localStorage.removeItem('refresh_token')
  }

  function setTokens(access: string, refresh: string) {
    accessToken.value = access
    refreshToken.value = refresh
    localStorage.setItem('access_token', access)
    localStorage.setItem('refresh_token', refresh)
  }

  return { user, accessToken, refreshToken, isAuthenticated, setup, login, fetchMe, logout }
})
