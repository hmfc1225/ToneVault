import client from './client'
import type { Book } from '@/types'

export async function searchBooks(query: string): Promise<Book[]> {
  const { data } = await client.get<Book[]>('/search', { params: { q: query } })
  return data
}