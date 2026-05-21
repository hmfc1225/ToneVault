<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { listAllUserBookmarks, deleteBookmark } from '@/api/bookmarks'
import { getBook } from '@/api/books'
import type { Bookmark, Book } from '@/types'

const router = useRouter()
const auth = useAuthStore()

const bookmarks = ref<(Bookmark & { bookTitle?: string })[]>([])
const loading = ref(true)

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.floor(secs % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

async function loadBookmarks() {
  if (!auth.user) { loading.value = false; return }
  loading.value = true
  try {
    const bms = await listAllUserBookmarks(auth.user.id)
    const withTitles = await Promise.all(
      bms.map(async (bm) => {
        try {
          const book = await getBook(bm.book_id)
          return { ...bm, bookTitle: book.title }
        } catch {
          return { ...bm, bookTitle: '未知' }
        }
      })
    )
    bookmarks.value = withTitles
  } catch { /* ignore */ } finally {
    loading.value = false
  }
}

async function onDelete(id: string) {
  try {
    await deleteBookmark(id)
    await loadBookmarks()
  } catch { /* ignore */ }
}

function goToBook(bookId: string) {
  router.push(`/books/${bookId}`)
}

onMounted(loadBookmarks)
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold" style="color: var(--text-primary)">书签</h1>

    <div v-if="loading" class="text-center py-8" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="bookmarks.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无书签</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="bm in bookmarks"
        :key="bm.id"
        class="p-4 rounded-lg flex items-center justify-between"
        style="background-color: var(--bg-card)"
      >
        <div class="flex-1 min-w-0 cursor-pointer" @click="goToBook(bm.book_id)">
          <h3 class="font-medium truncate" style="color: var(--text-primary)">{{ bm.title }}</h3>
          <p class="text-sm" style="color: var(--text-secondary)">
            {{ bm.bookTitle }} · {{ formatDuration(bm.position_secs) }}
          </p>
          <p v-if="bm.note" class="text-xs mt-1" style="color: var(--text-secondary)">{{ bm.note }}</p>
        </div>
        <button
          class="text-sm text-red-500 ml-3"
          @click="onDelete(bm.id)"
        >
          删除
        </button>
      </div>
    </div>
  </div>
</template>
