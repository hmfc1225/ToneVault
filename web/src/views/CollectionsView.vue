<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import * as collectionsApi from '@/api/collections'
import type { Collection, CreateCollection } from '@/types'

const auth = useAuthStore()
const router = useRouter()
const collections = ref<Collection[]>([])
const loading = ref(true)
const showCreate = ref(false)
const createForm = ref<CreateCollection>({ name: '' })

onMounted(async () => {
  if (!auth.user) return
  try {
    collections.value = await collectionsApi.listCollections(auth.user.id)
  } finally {
    loading.value = false
  }
})

async function createCollection() {
  if (!auth.user) return
  await collectionsApi.createCollection(auth.user.id, createForm.value)
  showCreate.value = false
  createForm.value = { name: '' }
  collections.value = await collectionsApi.listCollections(auth.user.id)
}

async function deleteCollection(id: string) {
  if (!confirm('确定要删除此收藏夹吗？')) return
  await collectionsApi.deleteCollection(id)
  collections.value = collections.value.filter(c => c.id !== id)
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold" style="color: var(--text-primary)">收藏夹</h1>
      <button @click="showCreate = true"
        class="px-4 py-2 rounded-lg text-sm font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors">
        新建收藏夹
      </button>
    </div>

    <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

    <div v-else-if="collections.length === 0" class="text-center py-12">
      <p style="color: var(--text-secondary)">暂无收藏夹，创建一个来整理你喜欢的有声书</p>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="col in collections" :key="col.id"
        class="p-5 rounded-xl border"
        style="background-color: var(--bg-card); border-color: var(--border-color)">
        <router-link :to="{ name: 'collection-detail', params: { id: col.id } }">
          <h3 class="font-semibold" style="color: var(--text-primary)">{{ col.name }}</h3>
        </router-link>
        <p v-if="col.description" class="text-sm mt-1" style="color: var(--text-secondary)">{{ col.description }}</p>
        <p class="text-xs mt-2" style="color: var(--text-tertiary)">{{ col.book_count }} 本有声书</p>
        <div class="mt-3 flex gap-2">
          <button @click="deleteCollection(col.id)"
            class="text-xs text-red-500 hover:text-red-700 transition-colors">删除</button>
        </div>
      </div>
    </div>

    <!-- Create Dialog -->
    <div v-if="showCreate" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="w-full max-w-md p-6 rounded-xl shadow-lg" style="background-color: var(--bg-card)">
        <h2 class="text-lg font-bold mb-4" style="color: var(--text-primary)">新建收藏夹</h2>
        <form @submit.prevent="createCollection" class="space-y-4">
          <div>
            <label class="block text-sm font-medium mb-1" style="color: var(--text-secondary)">名称</label>
            <input v-model="createForm.name" type="text" required
              class="w-full px-3 py-2 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500"
              style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)" />
          </div>
          <div>
            <label class="block text-sm font-medium mb-1" style="color: var(--text-secondary)">描述</label>
            <input v-model="createForm.description" type="text"
              class="w-full px-3 py-2 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500"
              style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)" />
          </div>
          <div class="flex gap-3">
            <button type="submit" class="px-4 py-2 rounded-lg font-medium bg-primary-600 text-white hover:bg-primary-700 transition-colors">创建</button>
            <button @click="showCreate = false" type="button" class="px-4 py-2 rounded-lg font-medium border transition-colors" style="color: var(--text-secondary); border-color: var(--border-color)">取消</button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>
