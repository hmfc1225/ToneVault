<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import * as booksApi from '@/api/books'
import * as authorsApi from '@/api/authors'
import * as seriesApi from '@/api/series'
import type { Book, Author, Series, PaginatedResult, BookFilter } from '@/types'

const route = useRoute()
const router = useRouter()

const books = ref<PaginatedResult<Book>>({ items: [], total: 0, page: 1, per_page: 20, total_pages: 0 })
const authors = ref<Author[]>([])
const seriesList = ref<Series[]>([])
const loading = ref(true)

const filter = ref<BookFilter>({
  library_id: route.query.library_id as string || undefined,
  author_id: route.query.author_id as string || undefined,
  series_id: route.query.series_id as string || undefined,
  query: route.query.q as string || undefined,
  sort: (route.query.sort as string || 'title') as BookFilter['sort'],
  order: (route.query.order as string || 'asc') as BookFilter['order'],
  page: Number(route.query.page) || 1,
  per_page: 20,
})

async function fetchBooks() {
  loading.value = true
  try {
    books.value = await booksApi.listBooks(filter.value)
  } finally {
    loading.value = false
  }
}

onMounted(async () => {
  authors.value = await authorsApi.listAuthors()
  seriesList.value = await seriesApi.listSeries()
  await fetchBooks()
})

watch(() => route.query, () => {
  filter.value = {
    library_id: route.query.library_id as string || undefined,
    author_id: route.query.author_id as string || undefined,
    series_id: route.query.series_id as string || undefined,
    query: route.query.q as string || undefined,
    sort: (route.query.sort as string || 'title') as BookFilter['sort'],
    order: (route.query.order as string || 'asc') as BookFilter['order'],
    page: Number(route.query.page) || 1,
    per_page: 20,
  }
  fetchBooks()
})

function updateFilter() {
  router.push({ name: 'books', query: { ...filter.value, q: filter.value.query } })
}

function goToPage(page: number) {
  filter.value.page = page
  updateFilter()
}

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
}

function coverUrl(book: Book): string {
  return booksApi.getBookCoverUrl(book.id)
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold mb-6" style="color: var(--text-primary)">有声书</h1>

    <!-- Filters -->
    <div class="flex flex-wrap gap-3 mb-6 p-4 rounded-xl border" style="background-color: var(--bg-card); border-color: var(--border-color)">
      <input v-model="filter.query" @keyup.enter="updateFilter" type="text" placeholder="搜索..."
        class="px-3 py-1.5 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500 text-sm w-48"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)" />

      <select v-model="filter.author_id" @change="updateFilter"
        class="px-3 py-1.5 rounded-lg border outline-none text-sm"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)">
        <option value="">全部作者</option>
        <option v-for="a in authors" :key="a.id" :value="a.id">{{ a.name }}</option>
      </select>

      <select v-model="filter.series_id" @change="updateFilter"
        class="px-3 py-1.5 rounded-lg border outline-none text-sm"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)">
        <option value="">全部系列</option>
        <option v-for="s in seriesList" :key="s.id" :value="s.id">{{ s.name }}</option>
      </select>

      <select v-model="filter.sort" @change="updateFilter"
        class="px-3 py-1.5 rounded-lg border outline-none text-sm"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)">
        <option value="title">标题</option>
        <option value="added">添加时间</option>
        <option value="duration">时长</option>
        <option value="year">年份</option>
        <option value="author">作者</option>
      </select>

      <select v-model="filter.order" @change="updateFilter"
        class="px-3 py-1.5 rounded-lg border outline-none text-sm"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)">
        <option value="asc">升序</option>
        <option value="desc">降序</option>
      </select>
    </div>

    <!-- Book Grid -->
    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="books.items.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无有声书</p>
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
      <router-link v-for="book in books.items" :key="book.id" :to="{ name: 'book-detail', params: { id: book.id } }"
        class="group rounded-xl border overflow-hidden hover:border-primary-500 transition-colors"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <div class="aspect-square bg-primary-100 dark:bg-primary-900 flex items-center justify-center overflow-hidden">
          <img v-if="book.cover_path" :src="coverUrl(book)" :alt="book.title" class="w-full h-full object-cover" loading="lazy" />
          <svg v-else class="w-12 h-12 text-primary-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.684 18 7.5 18s3.316.477 4.5 1.253m0-13C13.168 5.477 14.684 5 16.5 5c1.832 0 3.316.477 4.5 1.253v13C19.168 18.477 17.684 18 16.5 18c-1.832 0-3.316.477-4.5-1.253"/></svg>
        </div>
        <div class="p-3">
          <h3 class="font-medium text-sm truncate" style="color: var(--text-primary)">{{ book.title }}</h3>
          <p class="text-xs mt-1" style="color: var(--text-secondary)">{{ formatDuration(book.duration_secs) }}</p>
        </div>
      </router-link>
    </div>

    <!-- Pagination -->
    <div v-if="books.total_pages > 1" class="flex items-center justify-center gap-2 mt-6">
      <button @click="goToPage(books.page - 1)" :disabled="books.page <= 1"
        class="px-3 py-1.5 rounded-lg border text-sm disabled:opacity-50"
        style="color: var(--text-secondary); border-color: var(--border-color)">上一页</button>
      <span class="text-sm" style="color: var(--text-secondary)">第 {{ books.page }} / {{ books.total_pages }} 页</span>
      <button @click="goToPage(books.page + 1)" :disabled="books.page >= books.total_pages"
        class="px-3 py-1.5 rounded-lg border text-sm disabled:opacity-50"
        style="color: var(--text-secondary); border-color: var(--border-color)">下一页</button>
    </div>
  </div>
</template>
