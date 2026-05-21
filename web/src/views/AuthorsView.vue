<script setup lang="ts">
import { onMounted, ref } from 'vue'
import * as api from '@/api/authors'
import type { Author } from '@/types'

const authors = ref<Author[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    authors.value = await api.listAuthors()
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold mb-6" style="color: var(--text-primary)">作者</h1>

    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="authors.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无作者</p>
    </div>

    <div v-else class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
      <router-link v-for="author in authors" :key="author.id"
        :to="{ name: 'author-books', params: { id: author.id } }"
        class="p-4 rounded-xl border hover:border-primary-500 transition-colors text-center"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <div class="w-16 h-16 rounded-full mx-auto mb-3 flex items-center justify-center" style="background-color: var(--bg-secondary)">
          <span class="text-xl font-bold" style="color: var(--text-primary)">{{ author.name.charAt(0) }}</span>
        </div>
        <h3 class="font-medium text-sm" style="color: var(--text-primary)">{{ author.name }}</h3>
      </router-link>
    </div>
  </div>
</template>
