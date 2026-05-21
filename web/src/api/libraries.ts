import client from './client'
import type { Library, CreateLibrary, UpdateLibrary, ScanProgress, WebDavConnectRequest, WebDavEntry } from '@/types'

export async function listLibraries(): Promise<Library[]> {
  const { data } = await client.get<Library[]>('/libraries')
  return data
}

export async function getLibrary(id: string): Promise<Library> {
  const { data } = await client.get<Library>(`/libraries/${id}`)
  return data
}

export async function createLibrary(payload: CreateLibrary): Promise<Library> {
  const { data } = await client.post<Library>('/libraries', payload)
  return data
}

export async function updateLibrary(id: string, payload: UpdateLibrary): Promise<Library> {
  const { data } = await client.put<Library>(`/libraries/${id}`, payload)
  return data
}

export async function deleteLibrary(id: string): Promise<void> {
  await client.delete(`/libraries/${id}`)
}

export async function triggerScan(id: string): Promise<void> {
  await client.post(`/libraries/${id}/scan`)
}

export async function getScanStatus(id: string): Promise<ScanProgress> {
  const { data } = await client.get<ScanProgress>(`/libraries/${id}/scan/status`)
  return data
}

export async function webdavConnect(payload: WebDavConnectRequest): Promise<{ status: string; entries: WebDavEntry[] }> {
  const { data } = await client.post('/webdav/connect', payload)
  return data
}

export async function webdavList(payload: WebDavConnectRequest): Promise<WebDavEntry[]> {
  const { data } = await client.post<WebDavEntry[]>('/webdav/list', payload)
  return data
}