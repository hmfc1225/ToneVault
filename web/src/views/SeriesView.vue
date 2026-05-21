<script setup lang="ts">
import { onMounted, ref } from 'vue'
import * as api from '@/api/series'
import type { Series } from '@/types'

const seriesList = ref<Series[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    seriesList.value = await api.listSeries()
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold mb-6" style="color: var(--text-primary)">系列</h1>

    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="seriesList.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无系列</p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <router-link v-for="series in seriesList" :key="series.id"
        :to="{ name: 'series-books', params: { id: series.id } }"
        class="p-5 rounded-xl border hover:border-primary-500 transition-colors"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <h3 class="font-semibold" style="color: var(--text-primary)">{{ series.name }}</h3>
        <p v-if="series.description" class="text-sm mt-1" style="color: var(--text-secondary)">{{ series.description }}</p>
      </router-link>
    </div>
  </div>
</template>
