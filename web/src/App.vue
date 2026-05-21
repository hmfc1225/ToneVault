<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import AppShell from '@/components/layout/AppShell.vue'

const auth = useAuthStore()
const route = useRoute()

const showShell = computed(() => auth.isAuthenticated && !['login', 'setup'].includes(route.name as string))
</script>

<template>
  <AppShell v-if="showShell" />
  <router-view v-else />
</template>
