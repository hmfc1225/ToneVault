<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import * as seriesApi from '@/api/series'
import * as booksApi from '@/api/books'
import type { Book, PaginatedResult } from '@/types'

const route = useRoute()
const seriesId = route.params.id as string

const books = ref<PaginatedResult<Book>>({ items: [], total: 0, page: 1, per_page: 20, total_pages: 0 })
const loading = ref(true)

onMounted(async () => {
  books.value = await seriesApi.getSeriesBooks(seriesId)
  loading.value = false
})

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
}
</script>

<template>
  <div>
    <button @click="$router.push('/series')" class="text-sm mb-4 flex items-center gap-1" style="color: var(--text-secondary)">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>
      返回系列列表
    </button>
    <h1 class="text-2xl font-bold mb-6" style="color: var(--text-primary)">系列有声书</h1>

    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="books.items.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无有声书</p>
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      <router-link v-for="book in books.items" :key="book.id" :to="{ name: 'book-detail', params: { id: book.id } }"
        class="group rounded-xl border overflow-hidden hover:border-primary-500 transition-colors"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <div class="aspect-square bg-primary-100 dark:bg-primary-900 flex items-center justify-center overflow-hidden">
          <img v-if="book.cover_path" :src="booksApi.getBookCoverUrl(book.id)" :alt="book.title" class="w-full h-full object-cover" loading="lazy" />
          <svg v-else class="w-12 h-12 text-primary-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.684 18 7.5 18s3.316.477 4.5 1.253m0-13C13.168 5.477 14.684 5 16.5 5c1.832 0 3.316.477 4.5 1.253v13C19.168 18.477 17.684 18 16.5 18c-1.832 0-3.316.477-4.5-1.253"/></svg>
        </div>
        <div class="p-3">
          <h3 class="font-medium text-sm truncate" style="color: var(--text-primary)">{{ book.title }}</h3>
          <p class="text-xs mt-1" style="color: var(--text-secondary)">{{ formatDuration(book.duration_secs) }}</p>
        </div>
      </router-link>
    </div>
  </div>
</template>
