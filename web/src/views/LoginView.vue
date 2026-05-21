<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import client from '@/api/client'

const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)
const rememberMe = ref(false)
const loginBoxRef = ref<HTMLElement | null>(null)

const coverColors = [
  '#3b82f6', '#8b5cf6', '#ec4899', '#f59e0b', '#10b981',
  '#6366f1', '#14b8a6', '#f43f5e', '#a855f7', '#0ea5e9',
  '#22c55e', '#eab308', '#e11d48', '#7c3aed', '#06b6d4',
  '#84cc16', '#f97316', '#d946ef', '#2dd4bf', '#64748b',
]

const coverTitles = [
  '三体', '百年孤独', '活着', '围城', '红楼梦',
  '挪威的森林', '追风筝的人', '小王子', '1984', '白夜行',
  '解忧杂货店', '人间失格', '月亮与六便士', '了不起的盖茨比', '飘',
  '平凡的世界', '呐喊', '边城', '骆驼祥子', '雷雨',
]

interface CoverItem {
  color: string
  title: string
}

function shuffle<T>(arr: T[]): T[] {
  const copy = arr.slice()
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]]
  }
  return copy
}

function generateCovers(): CoverItem[] {
  return coverTitles.map((title, i) => ({
    color: coverColors[i % coverColors.length],
    title,
  }))
}

const row1 = ref<CoverItem[]>([])
const row2 = ref<CoverItem[]>([])
const row3 = ref<CoverItem[]>([])

onMounted(async () => {
  row1.value = shuffle(generateCovers())
  row2.value = shuffle(generateCovers())
  row3.value = shuffle(generateCovers())

  try {
    const { data } = await client.get('/auth/setup/status')
    if (data.needs_setup) {
      router.replace('/setup')
    }
  } catch {
    // If endpoint fails, just show login
  }

  const saved = localStorage.getItem('remember_auth')
  if (saved) {
    try {
      const parsed = JSON.parse(saved)
      username.value = parsed.username || ''
      if (parsed.password) {
        password.value = decodeURIComponent(atob(parsed.password))
      }
      rememberMe.value = true
    } catch {
      localStorage.removeItem('remember_auth')
    }
  }
})

async function onSubmit() {
  error.value = ''
  loading.value = true
  try {
    await auth.login(username.value, password.value)
    if (rememberMe.value) {
      localStorage.setItem('remember_auth', JSON.stringify({
        username: username.value,
        password: btoa(encodeURIComponent(password.value)),
      }))
    } else {
      localStorage.removeItem('remember_auth')
    }
    router.push('/')
  } catch (e: any) {
    error.value = e.response?.data?.error || e.response?.data?.message || '用户名或密码错误'
    loginBoxRef.value?.classList.remove('animate-shake')
    // Force reflow
    void loginBoxRef.value?.offsetWidth
    loginBoxRef.value?.classList.add('animate-shake')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex flex-col overflow-hidden transition-colors duration-500">
    <!-- Scrolling covers background -->
    <div class="flex-1 flex flex-col justify-center relative overflow-hidden bg-gray-50 dark:bg-gray-900 pb-10 transition-colors duration-500">
      <!-- Left fade -->
      <div class="absolute inset-y-0 left-0 w-20 bg-gradient-to-r from-white dark:from-gray-900 to-transparent z-10 transition-colors duration-500" />
      <!-- Right fade -->
      <div class="absolute inset-y-0 right-0 w-20 bg-gradient-to-l from-white dark:from-gray-900 to-transparent z-10 transition-colors duration-500" />

      <!-- Row 1: scroll left fast -->
      <div class="flex gap-6 mb-8 animate-scroll-left-fast w-max">
        <div
          v-for="(cover, idx) in [...row1, ...row1]"
          :key="`r1-${idx}`"
          class="w-32 h-32 rounded-lg shadow-lg overflow-hidden transform transition-transform hover:scale-105 flex items-center justify-center p-3"
          :style="{ backgroundColor: cover.color }"
        >
          <span class="text-white text-sm font-bold text-center leading-tight drop-shadow-md">{{ cover.title }}</span>
        </div>
      </div>

      <!-- Row 2: scroll right mid -->
      <div class="flex gap-6 mb-8 animate-scroll-right-mid w-max">
        <div
          v-for="(cover, idx) in [...row2, ...row2]"
          :key="`r2-${idx}`"
          class="w-32 h-32 rounded-lg shadow-lg overflow-hidden transform transition-transform hover:scale-105 flex items-center justify-center p-3"
          :style="{ backgroundColor: cover.color }"
        >
          <span class="text-white text-sm font-bold text-center leading-tight drop-shadow-md">{{ cover.title }}</span>
        </div>
      </div>

      <!-- Row 3: scroll left slow -->
      <div class="flex gap-6 animate-scroll-left-slow w-max">
        <div
          v-for="(cover, idx) in [...row3, ...row3]"
          :key="`r3-${idx}`"
          class="w-32 h-32 rounded-lg shadow-lg overflow-hidden transform transition-transform hover:scale-105 flex items-center justify-center p-3"
          :style="{ backgroundColor: cover.color }"
        >
          <span class="text-white text-sm font-bold text-center leading-tight drop-shadow-md">{{ cover.title }}</span>
        </div>
      </div>
    </div>

    <!-- Login card - overlaps the cover area -->
    <div
      ref="loginBoxRef"
      class="bg-white dark:bg-gray-800 rounded-t-4xl shadow-[0_-10px_40px_rgba(0,0,0,0.1)] dark:shadow-[0_-10px_40px_rgba(0,0,0,0.3)] py-4 px-8 pb-12 z-20 transition-colors duration-500 -mt-30"
    >
      <div class="max-w-md mx-auto animate-fade-in-up">
        <!-- Logo & Title -->
        <div class="text-center mb-8">
          <div class="w-16 h-16 rounded-xl mx-auto mb-4 shadow-lg shadow-blue-500/30 flex items-center justify-center bg-blue-600">
            <svg class="w-9 h-9 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          </div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 tracking-wide">TONEVAULT</h1>
          <p class="text-sm mt-1 text-gray-500 dark:text-gray-400">您的个人有声书库</p>
        </div>

        <!-- Login Form -->
        <form @submit.prevent="onSubmit" class="space-y-4">
          <!-- Username -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
            <input
              v-model="username"
              type="text"
              required
              autocomplete="username"
              placeholder="用户名"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
            />
          </div>

          <!-- Password -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </div>
            <input
              v-model="password"
              type="password"
              required
              autocomplete="current-password"
              placeholder="密码"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100"
            />
          </div>

          <!-- Error message -->
          <div v-if="error" class="text-red-500 text-sm text-center">{{ error }}</div>

          <!-- Remember me -->
          <div class="flex items-center justify-between">
            <label class="flex items-center space-x-2 text-sm text-gray-500 dark:text-gray-400 cursor-pointer">
              <input v-model="rememberMe" type="checkbox" class="rounded border-gray-300 text-blue-500 focus:ring-blue-500 dark:bg-gray-700" />
              <span>记住密码</span>
            </label>
          </div>

          <!-- Submit button -->
          <button
            type="submit"
            :disabled="loading"
            class="w-full py-3 px-4 rounded-xl font-bold text-white transition-all h-12 shadow-lg shadow-blue-500/30"
            :class="loading ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700 active:scale-[0.98]'"
          >
            {{ loading ? '登录中...' : '立即登录' }}
          </button>
        </form>

        <!-- Footer -->
        <div class="mt-8 text-center text-xs text-gray-400 dark:text-gray-500">
          <p>&copy; 2025 ToneVault. 保留所有权利。</p>
        </div>
      </div>
    </div>
  </div>
</template>
