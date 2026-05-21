import client from './client'
import type { Series, Book, PaginatedResult } from '@/types'

export async function listSeries(): Promise<Series[]> {
  const { data } = await client.get<Series[]>('/series')
  return data
}

export async function getSeriesBooks(id: string, page = 1, perPage = 20): Promise<PaginatedResult<Book>> {
  const { data } = await client.get<PaginatedResult<Book>>(`/series/${id}/books`, { params: { page, per_page: perPage } })
  return data
}