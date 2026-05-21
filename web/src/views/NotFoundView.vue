<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <div class="min-h-screen flex items-center justify-center" style="background-color: var(--bg-primary)">
    <div class="text-center">
      <svg class="w-24 h-24 mx-auto mb-6" style="color: var(--text-secondary)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <h1 class="text-4xl font-bold mb-2" style="color: var(--text-primary)">404</h1>
      <p class="text-lg mb-6" style="color: var(--text-secondary)">页面未找到</p>
      <button
        class="px-6 py-2 rounded-lg text-white font-medium"
        style="background-color: var(--color-primary)"
        @click="router.push('/')"
      >
        返回首页
      </button>
    </div>
  </div>
</template>
