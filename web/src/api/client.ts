import axios from 'axios'
import type { AuthResponse } from '@/types'

const client = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
})

client.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

let refreshPromise: Promise<string | null> | null = null

client.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config

    if (error.response?.status === 401 && !originalRequest._retry) {
      originalRequest._retry = true

      if (!refreshPromise) {
        refreshPromise = tryRefresh()
      }

      const newToken = await refreshPromise
      refreshPromise = null

      if (newToken) {
        originalRequest.headers.Authorization = `Bearer ${newToken}`
        return client(originalRequest)
      }

      localStorage.removeItem('access_token')
      localStorage.removeItem('refresh_token')
      window.location.href = '/login'
    }

    return Promise.reject(error)
  },
)

async function tryRefresh(): Promise<string | null> {
  const refreshToken = localStorage.getItem('refresh_token')
  if (!refreshToken) return null

  try {
    const { data } = await axios.post<AuthResponse>('/api/v1/auth/refresh', {
      refresh_token: refreshToken,
    })
    localStorage.setItem('access_token', data.access_token)
    localStorage.setItem('refresh_token', data.refresh_token)
    return data.access_token
  } catch {
    return null
  }
}

export default client
