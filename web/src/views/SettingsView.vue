<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import client from '@/api/client'

const auth = useAuthStore()
const isDark = ref(false)
const systemInfo = ref<any>(null)

onMounted(async () => {
  isDark.value = document.documentElement.classList.contains('dark')
  try {
    const { data } = await client.get('/system/info')
    systemInfo.value = data
  } catch { /* ignore */ }
})

function toggleDark() {
  isDark.value = !isDark.value
  document.documentElement.classList.toggle('dark', isDark.value)
  localStorage.setItem('theme', isDark.value ? 'dark' : 'light')
}

async function logout() {
  auth.logout()
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold" style="color: var(--text-primary)">设置</h1>

    <div class="space-y-4">
      <!-- User Info -->
      <div class="p-5 rounded-xl border" style="background-color: var(--bg-card); border-color: var(--border-color)">
        <h2 class="font-semibold mb-3" style="color: var(--text-primary)">账户</h2>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">用户名</span>
            <span style="color: var(--text-primary)">{{ auth.user?.username }}</span>
          </div>
          <div v-if="auth.user?.display_name" class="flex justify-between">
            <span style="color: var(--text-secondary)">显示名称</span>
            <span style="color: var(--text-primary)">{{ auth.user.display_name }}</span>
          </div>
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">角色</span>
            <span style="color: var(--text-primary)">{{ auth.user?.role === 'admin' ? '管理员' : auth.user?.role === 'user' ? '用户' : '访客' }}</span>
          </div>
        </div>
      </div>

      <!-- Theme -->
      <div class="p-5 rounded-xl border" style="background-color: var(--bg-card); border-color: var(--border-color)">
        <h2 class="font-semibold mb-3" style="color: var(--text-primary)">外观</h2>
        <div class="flex items-center justify-between">
          <span class="text-sm" style="color: var(--text-secondary)">深色模式</span>
          <button
            @click="toggleDark"
            class="relative w-12 h-6 rounded-full transition-colors"
            :style="{ backgroundColor: isDark ? 'var(--color-primary)' : 'var(--bg-tertiary)' }"
          >
            <span
              class="absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform"
              :class="isDark ? 'translate-x-6' : ''"
            />
          </button>
        </div>
      </div>

      <!-- System Info -->
      <div v-if="systemInfo" class="p-5 rounded-xl border" style="background-color: var(--bg-card); border-color: var(--border-color)">
        <h2 class="font-semibold mb-3" style="color: var(--text-primary)">系统信息</h2>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">版本</span>
            <span style="color: var(--text-primary)">{{ systemInfo.version }}</span>
          </div>
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">数据库</span>
            <span style="color: var(--text-primary)">{{ systemInfo.database_engine }}</span>
          </div>
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">书库数量</span>
            <span style="color: var(--text-primary)">{{ systemInfo.library_count }}</span>
          </div>
          <div class="flex justify-between">
            <span style="color: var(--text-secondary)">WebDAV</span>
            <span style="color: var(--text-primary)">{{ systemInfo.webdav_enabled ? '已启用' : '未启用' }}</span>
          </div>
        </div>
      </div>

      <!-- Logout -->
      <button
        @click="logout"
        class="w-full py-2.5 px-4 rounded-lg font-medium text-red-500 border border-red-300 hover:bg-red-50 dark:hover:bg-red-950 transition-colors"
      >
        退出登录
      </button>
    </div>
  </div>
</template>
