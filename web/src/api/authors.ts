import client from './client'
import type { Author, Book, PaginatedResult } from '@/types'

export async function listAuthors(): Promise<Author[]> {
  const { data } = await client.get<Author[]>('/authors')
  return data
}

export async function getAuthorBooks(id: string, page = 1, perPage = 20): Promise<PaginatedResult<Book>> {
  const { data } = await client.get<PaginatedResult<Book>>(`/authors/${id}/books`, { params: { page, per_page: perPage } })
  return data
}