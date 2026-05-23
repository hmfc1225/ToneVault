<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { createLibrary, webdavConnect, webdavList } from '@/api/libraries'
import type { SourceType, WebDavEntry } from '@/types'

const router = useRouter()

const activeTab = ref<SourceType>('local')
const loading = ref(false)
const error = ref('')

// Local mode fields
const localName = ref('')
const localPath = ref('')
const localWatch = ref(true)

// WebDAV mode fields
const webdavName = ref('')
const webdavUrl = ref('')
const webdavUsername = ref('')
const webdavPassword = ref('')
const webdavAutoSync = ref(false)
const webdavStep = ref<'connect' | 'browse'>('connect')
const webdavConnected = ref(false)
const webdavEntries = ref<WebDavEntry[]>([])
const webdavCurrentPath = ref('')
const webdavSelectedPath = ref('')
const webdavConnecting = ref(false)
const webdavBrowsing = ref(false)

// RSS mode fields
const rssName = ref('')
const rssUrl = ref('')
const rssAutoSync = ref(false)

const tabs = computed(() => [
  { key: 'local' as SourceType, label: '本地模式', icon: 'folder' },
  { key: 'webdav' as SourceType, label: 'WebDAV模式', icon: 'cloud' },
  { key: 'rss' as SourceType, label: 'RSS模式', icon: 'rss' },
])

const canSave = computed(() => {
  if (loading.value) return false
  switch (activeTab.value) {
    case 'local':
      return localName.value.trim() !== '' && localPath.value.trim() !== ''
    case 'webdav':
      return webdavName.value.trim() !== '' && (webdavSelectedPath.value.trim() !== '' || webdavCurrentPath.value.trim() !== '')
    case 'rss':
      return rssName.value.trim() !== '' && rssUrl.value.trim() !== ''
    default:
      return false
  }
})

const webdavBreadcrumbs = computed(() => {
  if (!webdavCurrentPath.value) return []
  // webdavCurrentPath is a full URL like https://host:port/dav/WebDAV
  // We want to show path segments after the origin
  const url = webdavCurrentPath.value
  const originMatch = url.match(/^(https?:\/\/[^/]+)(\/.*)?$/)
  if (!originMatch) return []
  const origin = originMatch[1]
  const pathPart = (originMatch[2] || '').replace(/\/$/, '')
  const originPath = webdavUrl.value.replace(/\/$/, '').match(/^https?:\/\/[^/]+(\/.*)?$/)
  const basePath = (originPath?.[1] || '').replace(/\/$/, '')
  // Only show segments after the base URL path
  let relPath = pathPart
  if (basePath && relPath.startsWith(basePath)) {
    relPath = relPath.slice(basePath.length)
  }
  const parts = relPath.split('/').filter(Boolean)
  return parts.map((part: string, idx: number) => ({
    label: decodeURIComponent(part),
    path: origin + basePath + '/' + parts.slice(0, idx + 1).join('/'),
  }))
})

function switchTab(tab: SourceType) {
  activeTab.value = tab
  error.value = ''
  if (tab === 'webdav') {
    webdavStep.value = 'connect'
    webdavConnected.value = false
    webdavEntries.value = []
    webdavCurrentPath.value = ''
    webdavSelectedPath.value = ''
  }
}

async function handleWebdavConnect() {
  error.value = ''
  webdavConnecting.value = true
  try {
    const result = await webdavConnect({
      url: webdavUrl.value,
      username: webdavUsername.value,
      password: webdavPassword.value,
    })
    webdavConnected.value = true
    webdavEntries.value = result.entries || []
    webdavCurrentPath.value = webdavUrl.value.trim()
    webdavStep.value = 'browse'
  } catch (e: any) {
    error.value = e.response?.data?.error || '无法连接到 WebDAV 服务器'
  } finally {
    webdavConnecting.value = false
  }
}

async function browseWebdavDir(path: string) {
  webdavBrowsing.value = true
  error.value = ''
  try {
    const entries = await webdavList({
      url: path,
      username: webdavUsername.value,
      password: webdavPassword.value,
    })
    webdavEntries.value = entries
    webdavCurrentPath.value = path
    webdavSelectedPath.value = ''
  } catch (e: any) {
    error.value = e.response?.data?.error || '无法浏览该目录'
  } finally {
    webdavBrowsing.value = false
  }
}

async function selectWebdavDir(entry: WebDavEntry) {
  webdavSelectedPath.value = entry.path
  // Don't auto-save, let user click save button
}

function goBackToConnect() {
  webdavStep.value = 'connect'
  webdavConnected.value = false
  webdavEntries.value = []
  webdavCurrentPath.value = ''
  webdavSelectedPath.value = ''
  error.value = ''
}

async function handleSave() {
  error.value = ''
  loading.value = true
  try {
    switch (activeTab.value) {
      case 'local':
        await createLibrary({
          name: localName.value,
          root_path: localPath.value,
          source_type: 'local' as SourceType,
        })
        break
      case 'webdav': {
        const rootPath = webdavSelectedPath.value || webdavCurrentPath.value
        await createLibrary({
          name: webdavName.value,
          root_path: rootPath,
          source_type: 'webdav',
          base_url: webdavUrl.value,
          username: webdavUsername.value,
          password: webdavPassword.value,
        })
        break
      }
      case 'rss':
        await createLibrary({
          name: rssName.value,
          root_path: rssUrl.value,
          source_type: 'rss',
          base_url: rssUrl.value,
        })
        break
    }
    router.push('/libraries')
  } catch (e: any) {
    error.value = e.response?.data?.error || '保存失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-50 dark:bg-gray-900 pb-24 transition-colors duration-500">
    <!-- Header -->
    <div class="sticky top-0 z-30 bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl border-b border-gray-100 dark:border-gray-800">
      <div class="max-w-2xl mx-auto px-4 py-4 flex items-center gap-3">
        <button
          @click="router.back()"
          class="w-10 h-10 rounded-xl flex items-center justify-center hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          <svg class="w-5 h-5 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <h1 class="text-lg font-bold text-gray-900 dark:text-gray-100">添加书库</h1>
      </div>
    </div>

    <div class="max-w-2xl mx-auto px-4 py-6 space-y-6">
      <!-- Mode Tabs -->
      <div class="bg-gray-100 dark:bg-gray-800 rounded-2xl p-1.5 flex gap-1">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          @click="switchTab(tab.key)"
          class="flex-1 flex items-center justify-center gap-2 py-2.5 px-3 rounded-xl text-sm font-medium transition-all duration-200"
          :class="activeTab === tab.key
            ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 shadow-sm'
            : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'"
        >
          <!-- Folder icon -->
          <svg v-if="tab.icon === 'folder'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <!-- Cloud icon -->
          <svg v-if="tab.icon === 'cloud'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          <!-- RSS icon -->
          <svg v-if="tab.icon === 'rss'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 5c7.18 0 13 5.82 13 13M6 11a7 7 0 017 7m-6 0a1 1 0 11-2 0 1 1 0 012 0z" />
          </svg>
          <span>{{ tab.label }}</span>
        </button>
      </div>

      <!-- Local Mode -->
      <div v-if="activeTab === 'local'" class="space-y-4 animate-fade-in-up">
        <div class="bg-white dark:bg-gray-800 rounded-3xl p-6 shadow-sm space-y-4">
          <!-- Name -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">书库名称</label>
            <input
              v-model="localName"
              type="text"
              placeholder="例如：我的有声书"
              class="w-full px-4 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
            />
          </div>
          <!-- Path -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">本地路径</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <input
                v-model="localPath"
                type="text"
                placeholder="/path/to/audiobooks"
                class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
              />
            </div>
            <p class="mt-1.5 text-xs text-gray-400 dark:text-gray-500">服务器上的音频文件目录路径</p>
          </div>
          <!-- Watch toggle -->
          <div class="flex items-center justify-between py-2">
            <div>
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">启用文件监控</p>
              <p class="text-xs text-gray-400 dark:text-gray-500">自动检测目录变化并更新书库</p>
            </div>
            <button
              @click="localWatch = !localWatch"
              class="relative w-12 h-7 rounded-full transition-colors duration-200"
              :class="localWatch ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
            >
              <span
                class="absolute top-0.5 left-0.5 w-6 h-6 bg-white rounded-full shadow transition-transform duration-200"
                :class="localWatch ? 'translate-x-5' : 'translate-x-0'"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- WebDAV Mode -->
      <div v-if="activeTab === 'webdav'" class="space-y-4 animate-fade-in-up">
        <!-- Step 1: Connect -->
        <div v-if="webdavStep === 'connect'" class="bg-white dark:bg-gray-800 rounded-3xl p-6 shadow-sm space-y-4">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-blue-50 dark:bg-blue-900/30 flex items-center justify-center">
              <svg class="w-5 h-5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
              </svg>
            </div>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">连接到 WebDAV 服务器</p>
              <p class="text-xs text-gray-400 dark:text-gray-500">输入服务器信息后连接并选择目录</p>
            </div>
          </div>

          <!-- Name -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">书库名称</label>
            <input
              v-model="webdavName"
              type="text"
              placeholder="例如：网盘有声书"
              class="w-full px-4 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
            />
          </div>
          <!-- URL -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">服务器地址</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
              </div>
              <input
                v-model="webdavUrl"
                type="url"
                placeholder="https://dav.example.com/path"
                class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
              />
            </div>
          </div>
          <!-- Username -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">用户名</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                </svg>
              </div>
              <input
                v-model="webdavUsername"
                type="text"
                placeholder="用户名"
                class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
              />
            </div>
          </div>
          <!-- Password -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">密码</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
              </div>
              <input
                v-model="webdavPassword"
                type="password"
                placeholder="密码"
                class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
              />
            </div>
          </div>

          <!-- Connect button -->
          <button
            @click="handleWebdavConnect"
            :disabled="webdavConnecting || !webdavUrl.trim() || !webdavUsername.trim() || !webdavPassword.trim()"
            class="w-full py-3 px-4 rounded-2xl font-bold text-white transition-all h-12 shadow-lg shadow-blue-500/30"
            :class="(webdavConnecting || !webdavUrl.trim() || !webdavUsername.trim() || !webdavPassword.trim()) ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700 active:scale-[0.98]'"
          >
            <span v-if="webdavConnecting" class="flex items-center justify-center gap-2">
              <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              连接中...
            </span>
            <span v-else>连接服务器</span>
          </button>
        </div>

        <!-- Step 2: Browse -->
        <div v-if="webdavStep === 'browse'" class="space-y-4">
          <!-- Connection info bar -->
          <div class="bg-white dark:bg-gray-800 rounded-3xl p-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <div class="w-8 h-8 rounded-lg bg-green-50 dark:bg-green-900/30 flex items-center justify-center">
                  <svg class="w-4 h-4 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </div>
                <div>
                  <p class="text-sm font-medium text-gray-900 dark:text-gray-100">已连接</p>
                  <p class="text-xs text-gray-400 dark:text-gray-500 truncate max-w-[200px]">{{ webdavUrl }}</p>
                </div>
              </div>
              <button
                @click="goBackToConnect"
                class="text-sm text-blue-600 dark:text-blue-400 hover:underline"
              >
                重新连接
              </button>
            </div>
          </div>

          <!-- Breadcrumbs -->
          <div v-if="webdavBreadcrumbs.length > 0" class="flex items-center gap-1 text-sm overflow-x-auto pb-1">
            <button
              @click="browseWebdavDir(webdavUrl)"
              class="text-blue-600 dark:text-blue-400 hover:underline whitespace-nowrap"
            >
              根目录
            </button>
            <template v-for="(crumb, idx) in webdavBreadcrumbs" :key="idx">
              <svg class="w-3 h-3 text-gray-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
              <button
                @click="browseWebdavDir(crumb.path)"
                class="text-blue-600 dark:text-blue-400 hover:underline whitespace-nowrap"
              >
                {{ crumb.label }}
              </button>
            </template>
          </div>

          <!-- Directory listing -->
          <div class="bg-white dark:bg-gray-800 rounded-3xl shadow-sm overflow-hidden">
            <div class="p-4 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between">
              <div>
                <p class="text-sm font-medium text-gray-700 dark:text-gray-300">选择书库目录</p>
                <p class="text-xs text-gray-400 dark:text-gray-500">点击文件夹进入，点击"选择"选中目录作为书库</p>
              </div>
              <button
                v-if="webdavCurrentPath"
                @click="webdavSelectedPath = webdavCurrentPath"
                class="px-3 py-1.5 text-xs rounded-lg bg-blue-600 text-white hover:bg-blue-700 transition-colors"
                :class="webdavSelectedPath === webdavCurrentPath ? 'opacity-50' : ''"
              >
                使用当前目录
              </button>
            </div>

            <!-- Loading -->
            <div v-if="webdavBrowsing" class="p-8 flex items-center justify-center">
              <svg class="w-6 h-6 animate-spin text-blue-600" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
            </div>

            <!-- Entries -->
            <div v-else-if="webdavEntries.length > 0" class="divide-y divide-gray-50 dark:divide-gray-700/50">
              <button
                v-for="entry in webdavEntries"
                :key="entry.path"
                @click="entry.is_dir ? browseWebdavDir(entry.path) : null"
                class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors"
                :class="webdavSelectedPath === entry.path ? 'bg-blue-50 dark:bg-blue-900/20' : ''"
              >
                <div
                  class="w-9 h-9 rounded-lg flex items-center justify-center"
                  :class="entry.is_dir ? 'bg-amber-50 dark:bg-amber-900/30' : 'bg-gray-50 dark:bg-gray-700'"
                >
                  <svg v-if="entry.is_dir" class="w-5 h-5 text-amber-500" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
                  </svg>
                  <svg v-else class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </div>
                <div class="flex-1 text-left">
                  <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ entry.name }}</p>
                  <p class="text-xs text-gray-400 dark:text-gray-500 truncate">{{ entry.path }}</p>
                </div>
                <svg v-if="entry.is_dir" class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
                <button
                  v-if="entry.is_dir"
                  @click.stop="selectWebdavDir(entry)"
                  class="ml-1 px-2 py-1 text-xs rounded-lg border border-blue-200 dark:border-blue-800 text-blue-600 dark:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30 transition-colors"
                >
                  选择
                </button>
                <svg v-if="webdavSelectedPath === entry.path" class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </button>
            </div>

            <!-- Empty -->
            <div v-else class="p-8 text-center">
              <p class="text-sm text-gray-400 dark:text-gray-500">该目录为空</p>
            </div>
          </div>

          <!-- Auto sync toggle -->
          <div class="bg-white dark:bg-gray-800 rounded-3xl p-4 shadow-sm">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium text-gray-700 dark:text-gray-300">自动同步</p>
                <p class="text-xs text-gray-400 dark:text-gray-500">定期从 WebDAV 同步新书</p>
              </div>
              <button
                @click="webdavAutoSync = !webdavAutoSync"
                class="relative w-12 h-7 rounded-full transition-colors duration-200"
                :class="webdavAutoSync ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
              >
                <span
                  class="absolute top-0.5 left-0.5 w-6 h-6 bg-white rounded-full shadow transition-transform duration-200"
                  :class="webdavAutoSync ? 'translate-x-5' : 'translate-x-0'"
                />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- RSS Mode -->
      <div v-if="activeTab === 'rss'" class="space-y-4 animate-fade-in-up">
        <div class="bg-white dark:bg-gray-800 rounded-3xl p-6 shadow-sm space-y-4">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-orange-50 dark:bg-orange-900/30 flex items-center justify-center">
              <svg class="w-5 h-5 text-orange-600 dark:text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 5c7.18 0 13 5.82 13 13M6 11a7 7 0 017 7m-6 0a1 1 0 11-2 0 1 1 0 012 0z" />
              </svg>
            </div>
            <div>
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100">RSS 订阅源</p>
              <p class="text-xs text-gray-400 dark:text-gray-500">从 RSS 订阅源自动获取有声书</p>
            </div>
          </div>

          <!-- Name -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">书库名称</label>
            <input
              v-model="rssName"
              type="text"
              placeholder="例如：播客订阅"
              class="w-full px-4 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
            />
          </div>
          <!-- RSS URL -->
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">RSS 订阅地址</label>
            <div class="relative">
              <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
              </div>
              <input
                v-model="rssUrl"
                type="url"
                placeholder="https://example.com/feed.xml"
                class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
              />
            </div>
            <p class="mt-1.5 text-xs text-gray-400 dark:text-gray-500">支持 RSS 2.0 和 Atom 格式的订阅源</p>
          </div>

          <!-- Auto sync toggle -->
          <div class="flex items-center justify-between py-2">
            <div>
              <p class="text-sm font-medium text-gray-700 dark:text-gray-300">自动同步</p>
              <p class="text-xs text-gray-400 dark:text-gray-500">定期检查 RSS 更新并下载新书</p>
            </div>
            <button
              @click="rssAutoSync = !rssAutoSync"
              class="relative w-12 h-7 rounded-full transition-colors duration-200"
              :class="rssAutoSync ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'"
            >
              <span
                class="absolute top-0.5 left-0.5 w-6 h-6 bg-white rounded-full shadow transition-transform duration-200"
                :class="rssAutoSync ? 'translate-x-5' : 'translate-x-0'"
              />
            </button>
          </div>
        </div>
      </div>

      <!-- Error -->
      <div v-if="error" class="bg-red-50 dark:bg-red-900/20 rounded-2xl p-4 flex items-center gap-3">
        <svg class="w-5 h-5 text-red-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-sm text-red-600 dark:text-red-400">{{ error }}</p>
      </div>
    </div>

    <!-- Bottom fixed action bar -->
    <div class="fixed bottom-0 left-0 right-0 bg-white/80 dark:bg-gray-900/80 backdrop-blur-xl border-t border-gray-100 dark:border-gray-800 z-30">
      <div class="max-w-2xl mx-auto px-4 py-4">
        <button
          @click="handleSave"
          :disabled="!canSave"
          class="w-full py-3 px-4 rounded-2xl font-bold text-white transition-all h-12 shadow-lg shadow-blue-500/30"
          :class="!canSave ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700 active:scale-[0.98]'"
        >
          <span v-if="loading" class="flex items-center justify-center gap-2">
            <svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            保存中...
          </span>
          <span v-else>保存</span>
        </button>
      </div>
    </div>
  </div>
</template>
