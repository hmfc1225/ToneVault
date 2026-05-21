<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const displayName = ref('')
const error = ref('')
const loading = ref(false)

async function onSubmit() {
  error.value = ''
  if (password.value !== confirmPassword.value) {
    error.value = '两次输入的密码不一致'
    return
  }
  if (password.value.length < 6) {
    error.value = '密码至少需要6个字符'
    return
  }
  loading.value = true
  try {
    await auth.setup(username.value, password.value, displayName.value || undefined, email.value || undefined)
    router.push('/')
  } catch (e: any) {
    error.value = e.response?.data?.error || e.response?.data?.message || '创建失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex flex-col overflow-hidden transition-colors duration-500">
    <!-- Decorative background -->
    <div class="flex-1 flex flex-col justify-center relative overflow-hidden bg-gray-50 dark:bg-gray-900 pb-10 transition-colors duration-500">
      <!-- Left fade -->
      <div class="absolute inset-y-0 left-0 w-20 bg-gradient-to-r from-white dark:from-gray-900 to-transparent z-10 transition-colors duration-500" />
      <!-- Right fade -->
      <div class="absolute inset-y-0 right-0 w-20 bg-gradient-to-l from-white dark:from-gray-900 to-transparent z-10 transition-colors duration-500" />

      <!-- Decorative circles -->
      <div class="absolute inset-0 overflow-hidden">
        <div class="absolute -top-20 -left-20 w-72 h-72 bg-blue-100 dark:bg-blue-900/30 rounded-full opacity-50" />
        <div class="absolute -bottom-16 -right-16 w-64 h-64 bg-purple-100 dark:bg-purple-900/30 rounded-full opacity-50" />
        <div class="absolute top-1/3 right-1/4 w-40 h-40 bg-green-100 dark:bg-green-900/20 rounded-full opacity-40" />
      </div>

      <!-- Center icon -->
      <div class="relative z-10 flex flex-col items-center justify-center">
        <div class="w-20 h-20 rounded-2xl shadow-lg shadow-blue-500/30 flex items-center justify-center bg-blue-600 mb-4">
          <svg class="w-11 h-11 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
          </svg>
        </div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100 tracking-wide">TONEVAULT</h1>
        <p class="text-sm mt-1 text-gray-500 dark:text-gray-400">初始设置 · 创建管理员账号</p>
      </div>
    </div>

    <!-- Setup card -->
    <div class="bg-white dark:bg-gray-800 rounded-t-4xl shadow-[0_-10px_40px_rgba(0,0,0,0.1)] dark:shadow-[0_-10px_40px_rgba(0,0,0,0.3)] py-4 px-8 pb-12 z-20 transition-colors duration-500 -mt-16">
      <div class="max-w-md mx-auto animate-fade-in-up">
        <form @submit.prevent="onSubmit" class="space-y-4">
          <!-- Username -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
              </svg>
            </div>
            <input v-model="username" type="text" required autocomplete="username" placeholder="用户名"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100" />
          </div>

          <!-- Email -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
            </div>
            <input v-model="email" type="email" required autocomplete="email" placeholder="邮箱"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100" />
          </div>

          <!-- Display Name -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.121 17.804A13.937 13.937 0 0112 16c2.5 0 4.847.655 6.879 1.804M15 10a3 3 0 11-6 0 3 3 0 016 0zm6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <input v-model="displayName" type="text" autocomplete="name" placeholder="显示名称（可选）"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100" />
          </div>

          <!-- Password -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
            </div>
            <input v-model="password" type="password" required autocomplete="new-password" placeholder="密码"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100" />
          </div>

          <!-- Confirm Password -->
          <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
              <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
            </div>
            <input v-model="confirmPassword" type="password" required autocomplete="new-password" placeholder="确认密码"
              class="w-full pl-10 pr-3 py-3 rounded-xl border border-gray-100 dark:border-gray-600 bg-gray-50 dark:bg-gray-700 outline-none focus:bg-white dark:focus:bg-gray-600 focus:ring-2 focus:ring-blue-500 transition-colors text-gray-900 dark:text-gray-100" />
          </div>

          <!-- Error message -->
          <div v-if="error" class="text-red-500 text-sm text-center">{{ error }}</div>

          <!-- Submit button -->
          <button
            type="submit"
            :disabled="loading"
            class="w-full py-3 px-4 rounded-xl font-bold text-white transition-all h-12 shadow-lg shadow-blue-500/30"
            :class="loading ? 'bg-blue-400 cursor-not-allowed' : 'bg-blue-600 hover:bg-blue-700 active:scale-[0.98]'"
          >
            {{ loading ? '创建中...' : '创建管理员' }}
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
