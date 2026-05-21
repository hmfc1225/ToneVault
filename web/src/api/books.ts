import client from './client'
import type { Book, Track, AuthorWithRole, SeriesWithPosition, PaginatedResult, BookFilter } from '@/types'

export async function listBooks(filter?: BookFilter): Promise<PaginatedResult<Book>> {
  const params: Record<string, string | number> = {}
  if (filter?.library_id) params.library_id = filter.library_id
  if (filter?.author_id) params.author_id = filter.author_id
  if (filter?.series_id) params.series_id = filter.series_id
  if (filter?.query) params.query = filter.query
  if (filter?.sort) params.sort = filter.sort
  if (filter?.order) params.order = filter.order
  if (filter?.page) params.page = filter.page
  if (filter?.per_page) params.per_page = filter.per_page
  const { data } = await client.get<PaginatedResult<Book>>('/books', { params })
  return data
}

export async function getBook(id: string): Promise<Book> {
  const { data } = await client.get<Book>(`/books/${id}`)
  return data
}

export async function updateBook(id: string, payload: Partial<Book>): Promise<Book> {
  const { data } = await client.put<Book>(`/books/${id}`, payload)
  return data
}

export async function deleteBook(id: string): Promise<void> {
  await client.delete(`/books/${id}`)
}

export async function getBookTracks(id: string): Promise<Track[]> {
  const { data } = await client.get<Track[]>(`/books/${id}/tracks`)
  return data
}

export async function getBookAuthors(id: string): Promise<AuthorWithRole[]> {
  const { data } = await client.get<AuthorWithRole[]>(`/books/${id}/authors`)
  return data
}

export async function getBookSeries(id: string): Promise<SeriesWithPosition[]> {
  const { data } = await client.get<SeriesWithPosition[]>(`/books/${id}/series`)
  return data
}

export function getBookCoverUrl(id: string): string {
  return `/api/v1/books/${id}/cover`
}

export function getTrackStreamUrl(id: string): string {
  return `/api/v1/tracks/${id}/stream`
}