<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import * as booksApi from '@/api/books'
import type { Book } from '@/types'

const route = useRoute()
const router = useRouter()

const query = ref((route.query.q as string) || '')
const books = ref<Book[]>([])
const loading = ref(false)

async function search() {
  if (!query.value.trim()) { books.value = []; return }
  loading.value = true
  try {
    const result = await booksApi.searchBooks(query.value)
    books.value = result
  } catch { /* ignore */ } finally {
    loading.value = false
  }
}

watch(() => route.query.q, (q) => {
  query.value = (q as string) || ''
  if (query.value) search()
}, { immediate: true })

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold mb-6" style="color: var(--text-primary)">搜索</h1>

    <div class="flex gap-2 mb-6">
      <input v-model="query" @keyup.enter="search" type="text" placeholder="搜索有声书、作者、系列..."
        class="flex-1 px-4 py-2.5 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)" />
      <button @click="search"
        class="px-6 py-2.5 rounded-lg font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors">
        搜索
      </button>
    </div>

    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">搜索中...</div>

    <div v-else-if="query && books.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">未找到相关结果</p>
    </div>

    <div v-else-if="books.length > 0" class="space-y-3">
      <p class="text-sm" style="color: var(--text-secondary)">找到 {{ books.length }} 个结果</p>
      <router-link v-for="book in books" :key="book.id" :to="`/books/${book.id}`"
        class="block p-4 rounded-xl border hover:border-primary-500 transition-colors"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <div class="flex gap-4">
          <div class="w-12 h-16 rounded flex-shrink-0 flex items-center justify-center" style="background-color: var(--bg-secondary)">
            <svg class="w-6 h-6" style="color: var(--text-secondary)" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.684 18 7.5 18s3.316.477 4.5 1.253m0-13C13.168 5.477 14.684 5 16.5 5c1.832 0 3.316.477 4.5 1.253v13C19.168 18.477 17.684 18 16.5 18c-1.832 0-3.316.477-4.5-1.253"/></svg>
          </div>
          <div class="flex-1 min-w-0">
            <h3 class="font-medium" style="color: var(--text-primary)">{{ book.title }}</h3>
            <p class="text-sm" style="color: var(--text-secondary)">{{ book.authors?.map(a => a.name).join(', ') || '未知作者' }}</p>
            <p v-if="book.duration_secs" class="text-xs mt-1" style="color: var(--text-tertiary)">{{ formatDuration(book.duration_secs) }}</p>
          </div>
        </div>
      </router-link>
    </div>
  </div>
</template>
