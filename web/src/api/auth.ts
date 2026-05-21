import client from './client'
import type { AuthResponse, User } from '@/types'

export async function setup(username: string, password: string, display_name?: string, email?: string): Promise<AuthResponse> {
  const { data } = await client.post<AuthResponse>('/auth/setup', { username, password, display_name, email: email || '' })
  return data
}

export async function login(username: string, password: string): Promise<AuthResponse> {
  const { data } = await client.post<AuthResponse>('/auth/login', { username, password })
  return data
}

export async function refresh(refresh_token: string): Promise<AuthResponse> {
  const { data } = await client.post<AuthResponse>('/auth/refresh', { refresh_token })
  return data
}

export async function getMe(): Promise<User> {
  const { data } = await client.get<User>('/auth/me')
  return data
}
