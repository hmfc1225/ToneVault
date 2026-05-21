import client from './client'
import type { Collection, CreateCollection, Book } from '@/types'

export async function listCollections(userId: string): Promise<Collection[]> {
  const { data } = await client.get<Collection[]>(`/collections/${userId}`)
  return data
}

export async function createCollection(userId: string, payload: CreateCollection): Promise<Collection> {
  const { data } = await client.post<Collection>(`/collections/${userId}`, payload)
  return data
}

export async function deleteCollection(id: string): Promise<void> {
  await client.delete(`/collections/item/${id}`)
}

export async function addBookToCollection(collectionId: string, bookId: string): Promise<void> {
  await client.post(`/collections/${collectionId}/books/${bookId}`)
}

export async function removeBookFromCollection(collectionId: string, bookId: string): Promise<void> {
  await client.delete(`/collections/${collectionId}/books/${bookId}`)
}

export async function getCollectionBooks(id: string): Promise<Book[]> {
  const { data } = await client.get<Book[]>(`/collections/${id}/books`)
  return data
}