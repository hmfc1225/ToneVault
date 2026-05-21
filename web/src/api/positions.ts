import client from './client'
import type { PlaybackPosition, UpsertPlaybackPosition } from '@/types'

export async function upsertPosition(userId: string, payload: UpsertPlaybackPosition): Promise<PlaybackPosition> {
  const { data } = await client.put<PlaybackPosition>(`/positions/${userId}`, payload)
  return data
}

export async function getUserPositions(userId: string): Promise<PlaybackPosition[]> {
  const { data } = await client.get<PlaybackPosition[]>(`/positions/${userId}`)
  return data
}

export async function getPosition(userId: string, bookId: string): Promise<PlaybackPosition | null> {
  const { data } = await client.get<PlaybackPosition>(`/positions/${userId}/${bookId}`)
  return data
}