<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { listLibraries, deleteLibrary, triggerScan } from '@/api/libraries'
import type { Library, SourceType } from '@/types'

const router = useRouter()
const libraries = ref<Library[]>([])
const loading = ref(true)
const scanning = ref<string | null>(null)

const sourceLabels: Record<SourceType, string> = {
  local: '本地',
  webdav: 'WebDAV',
  rss: 'RSS',
}

const sourceColors: Record<SourceType, string> = {
  local: 'bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300',
  webdav: 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400',
  rss: 'bg-orange-100 text-orange-600 dark:bg-orange-900/30 dark:text-orange-400',
}

onMounted(async () => {
  await loadLibraries()
})

async function loadLibraries() {
  loading.value = true
  try {
    libraries.value = await listLibraries()
  } catch {
    libraries.value = []
  } finally {
    loading.value = false
  }
}

async function handleScan(id: string) {
  scanning.value = id
  try {
    await triggerScan(id)
    await loadLibraries()
  } catch {
    // ignore
  } finally {
    scanning.value = null
  }
}

async function handleDelete(id: string) {
  if (!confirm('确定要删除此书库吗？此操作不可撤销。')) return
  try {
    await deleteLibrary(id)
    await loadLibraries()
  } catch {
    // ignore
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-500">
    <!-- Header -->
    <div class="sticky top-0 z-30 bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl border-b border-gray-100 dark:border-gray-800">
      <div class="max-w-4xl mx-auto px-4 py-4 flex items-center justify-between">
        <h1 class="text-lg font-bold text-gray-900 dark:text-gray-100">书库管理</h1>
        <button
          @click="router.push('/libraries/add')"
          class="flex items-center gap-2 px-4 py-2.5 rounded-2xl bg-blue-600 text-white font-medium text-sm shadow-lg shadow-blue-500/30 hover:bg-blue-700 active:scale-[0.98] transition-all"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          添加书库
        </button>
      </div>
    </div>

    <div class="max-w-4xl mx-auto px-4 py-6">
      <!-- Loading -->
      <div v-if="loading" class="flex items-center justify-center py-20">
        <svg class="w-8 h-8 animate-spin text-blue-600" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      </div>

      <!-- Empty state -->
      <div v-else-if="libraries.length === 0" class="text-center py-20">
        <div class="w-20 h-20 rounded-2xl bg-gray-100 dark:bg-gray-800 flex items-center justify-center mx-auto mb-4">
          <svg class="w-10 h-10 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <p class="text-gray-500 dark:text-gray-400 mb-2">还没有书库</p>
        <p class="text-sm text-gray-400 dark:text-gray-500 mb-6">添加一个书库开始管理您的有声书</p>
        <button
          @click="router.push('/libraries/add')"
          class="inline-flex items-center gap-2 px-6 py-3 rounded-2xl bg-blue-600 text-white font-medium shadow-lg shadow-blue-500/30 hover:bg-blue-700 active:scale-[0.98] transition-all"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          添加书库
        </button>
      </div>

      <!-- Library list -->
      <div v-else class="space-y-3">
        <div
          v-for="lib in libraries"
          :key="lib.id"
          class="bg-white dark:bg-gray-800 rounded-3xl p-5 shadow-sm hover:shadow-md transition-shadow"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="font-bold text-gray-900 dark:text-gray-100 truncate">{{ lib.name }}</h3>
                <span
                  class="inline-flex items-center px-2 py-0.5 rounded-lg text-xs font-medium"
                  :class="sourceColors[lib.source_type || 'local']"
                >
                  {{ sourceLabels[lib.source_type || 'local'] }}
                </span>
              </div>
              <p class="text-sm text-gray-500 dark:text-gray-400 truncate">{{ lib.root_path }}</p>
              <div v-if="lib.base_url && lib.source_type !== 'local'" class="mt-1">
                <p class="text-xs text-gray-400 dark:text-gray-500 truncate">{{ lib.base_url }}</p>
              </div>
              <div class="flex items-center gap-4 mt-2">
                <span class="text-xs text-gray-400 dark:text-gray-500">
                  {{ lib.book_count ?? 0 }} 本书
                </span>
                <span v-if="lib.last_scan" class="text-xs text-gray-400 dark:text-gray-500">
                  上次扫描: {{ new Date(lib.last_scan).toLocaleDateString() }}
                </span>
              </div>
            </div>
            <div class="flex items-center gap-2 ml-4">
              <button
                @click="handleScan(lib.id)"
                :disabled="scanning === lib.id"
                class="w-9 h-9 rounded-xl flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                title="扫描书库"
              >
                <svg v-if="scanning === lib.id" class="w-4 h-4 animate-spin text-blue-600" fill="none" viewBox="0 0 24 24">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                <svg v-else class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
              </button>
              <button
                @click="handleDelete(lib.id)"
                class="w-9 h-9 rounded-xl flex items-center justify-center hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
                title="删除书库"
              >
                <svg class="w-4 h-4 text-gray-500 hover:text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
