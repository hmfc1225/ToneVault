<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()
const mobileMenuOpen = ref(false)
const userMenuOpen = ref(false)

const navItems = [
  { path: '/', label: '首页', icon: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6' },
  { path: '/books', label: '有声书', icon: 'M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253' },
  { path: '/authors', label: '作者', icon: 'M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z' },
  { path: '/series', label: '系列', icon: 'M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10' },
  { path: '/collections', label: '收藏夹', icon: 'M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z' },
  { path: '/bookmarks', label: '书签', icon: 'M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z' },
  { path: '/libraries', label: '书库', icon: 'M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z' },
  { path: '/search', label: '搜索', icon: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z' },
]

function isActive(path: string): boolean {
  if (path === '/') return router.currentRoute.value.path === '/'
  return router.currentRoute.value.path.startsWith(path)
}

function navigate(path: string) {
  router.push(path)
  mobileMenuOpen.value = false
}

function logout() {
  auth.logout()
  router.push('/login')
}
</script>

<template>
  <div class="flex h-screen" style="background-color: var(--bg-primary)">
    <!-- Desktop Sidebar -->
    <aside class="hidden md:flex flex-col w-56 border-r" style="background-color: var(--bg-card); border-color: var(--border-color)">
      <div class="p-4 border-b" style="border-color: var(--border-color)">
        <h1 class="text-xl font-bold" style="color: var(--text-primary)">ToneVault</h1>
      </div>
      <nav class="flex-1 p-2 space-y-1">
        <button
          v-for="item in navItems"
          :key="item.path"
          @click="navigate(item.path)"
          class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors"
          :style="{
            backgroundColor: isActive(item.path) ? 'var(--bg-secondary)' : 'transparent',
            color: isActive(item.path) ? 'var(--color-primary)' : 'var(--text-secondary)'
          }"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="item.icon" />
          </svg>
          {{ item.label }}
        </button>
      </nav>
      <div class="p-3 border-t" style="border-color: var(--border-color)">
        <div class="flex items-center gap-2 px-3 py-2">
          <div class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium text-white" style="background-color: var(--color-primary)">
            {{ auth.user?.username?.charAt(0).toUpperCase() }}
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium truncate" style="color: var(--text-primary)">{{ auth.user?.display_name || auth.user?.username }}</p>
            <p class="text-xs" style="color: var(--text-secondary)">{{ auth.user?.role === 'admin' ? '管理员' : '用户' }}</p>
          </div>
          <button @click="logout" class="p-1 rounded hover:opacity-70" style="color: var(--text-secondary)">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/></svg>
          </button>
        </div>
      </div>
    </aside>

    <!-- Mobile Header -->
    <div class="md:hidden fixed top-0 left-0 right-0 z-40 border-b" style="background-color: var(--bg-card); border-color: var(--border-color)">
      <div class="flex items-center justify-between px-4 py-3">
        <h1 class="text-lg font-bold" style="color: var(--text-primary)">ToneVault</h1>
        <button @click="mobileMenuOpen = !mobileMenuOpen" class="p-1" style="color: var(--text-primary)">
          <svg v-if="!mobileMenuOpen" class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
          <svg v-else class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
        </button>
      </div>
    </div>

    <!-- Mobile Menu Overlay -->
    <div v-if="mobileMenuOpen" class="md:hidden fixed inset-0 z-30 bg-black/50" @click="mobileMenuOpen = false">
      <div class="w-64 h-full p-4 space-y-1" style="background-color: var(--bg-card)" @click.stop>
        <button
          v-for="item in navItems"
          :key="item.path"
          @click="navigate(item.path)"
          class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors"
          :style="{
            backgroundColor: isActive(item.path) ? 'var(--bg-secondary)' : 'transparent',
            color: isActive(item.path) ? 'var(--color-primary)' : 'var(--text-secondary)'
          }"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="item.icon" />
          </svg>
          {{ item.label }}
        </button>
        <div class="border-t my-3" style="border-color: var(--border-color)"></div>
        <button @click="logout" class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium text-red-500">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/></svg>
          退出登录
        </button>
      </div>
    </div>

    <!-- Main Content -->
    <main class="flex-1 overflow-auto md:pt-0 pt-14">
      <div class="p-6">
        <router-view />
      </div>
    </main>
  </div>
</template>
