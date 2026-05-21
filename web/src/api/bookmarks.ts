import client from './client'
import type { Bookmark, CreateBookmark } from '@/types'

export async function createBookmark(userId: string, payload: CreateBookmark): Promise<Bookmark> {
  const { data } = await client.post<Bookmark>(`/bookmarks/${userId}`, payload)
  return data
}

export async function listBookmarks(userId: string, bookId: string): Promise<Bookmark[]> {
  const { data } = await client.get<Bookmark[]>(`/bookmarks/${userId}/${bookId}`)
  return data
}

export async function listAllUserBookmarks(userId: string): Promise<Bookmark[]> {
  const { data } = await client.get<Bookmark[]>(`/bookmarks/${userId}`)
  return data
}

export async function deleteBookmark(id: string): Promise<void> {
  await client.delete(`/bookmarks/item/${id}`)
}