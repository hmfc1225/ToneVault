<script setup lang="ts">
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { getBookCoverUrl } from '@/api/books'

const player = useAudioPlayer()

const speeds = [0.5, 0.75, 1, 1.25, 1.5, 2, 3]
const showSpeedMenu = ref(false)

function formatTime(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.floor(secs % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function onProgressClick(e: MouseEvent) {
  const bar = e.currentTarget as HTMLElement
  const rect = bar.getBoundingClientRect()
  const percent = (e.clientX - rect.left) / rect.width
  player.seekByPercent(percent)
}

import { ref } from 'vue'
</script>

<template>
  <div v-if="player.currentBook.value" class="h-20 border-t flex items-center px-4 gap-4" style="background-color: var(--bg-secondary); border-color: var(--border-color)">
    <!-- Cover + Info -->
    <div class="flex items-center gap-3 w-64 shrink-0">
      <div class="w-12 h-12 rounded-lg overflow-hidden bg-primary-100 dark:bg-primary-900 shrink-0">
        <img v-if="player.currentBook.value.cover_path" :src="getBookCoverUrl(player.currentBook.value.id)" class="w-full h-full object-cover" />
      </div>
      <div class="min-w-0">
        <p class="text-sm font-medium truncate" style="color: var(--text-primary)">{{ player.currentBook.value.title }}</p>
        <p class="text-xs truncate" style="color: var(--text-secondary)">{{ player.currentTrack.value?.title }}</p>
      </div>
    </div>

    <!-- Controls + Progress -->
    <div class="flex-1 flex flex-col items-center gap-1">
      <div class="flex items-center gap-3">
        <button @click="player.playPrev" class="p-1.5 rounded-full hover:bg-primary-100 dark:hover:bg-primary-900 transition-colors" style="color: var(--text-secondary)">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
        </button>
        <button @click="player.togglePlay"
          class="w-10 h-10 rounded-full bg-primary-600 text-white flex items-center justify-center hover:bg-primary-700 transition-colors">
          <svg v-if="player.isPlaying.value" class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
          <svg v-else class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
        </button>
        <button @click="player.playNext" class="p-1.5 rounded-full hover:bg-primary-100 dark:hover:bg-primary-900 transition-colors" style="color: var(--text-secondary)">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
        </button>
      </div>
      <div class="w-full flex items-center gap-2">
        <span class="text-xs w-12 text-right tabular-nums" style="color: var(--text-tertiary)">{{ formatTime(player.currentTime.value) }}</span>
        <div @click="onProgressClick" class="flex-1 h-1.5 rounded-full cursor-pointer group" style="background-color: var(--bg-tertiary)">
          <div class="h-1.5 rounded-full bg-primary-500 group-hover:bg-primary-400 transition-all relative"
            :style="{ width: `${player.duration.value > 0 ? (player.currentTime.value / player.duration.value) * 100 : 0}%` }">
            <div class="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 rounded-full bg-primary-500 opacity-0 group-hover:opacity-100 transition-opacity"></div>
          </div>
        </div>
        <span class="text-xs w-12 tabular-nums" style="color: var(--text-tertiary)">{{ formatTime(player.duration.value) }}</span>
      </div>
    </div>

    <!-- Speed + Volume -->
    <div class="flex items-center gap-2 w-32 justify-end shrink-0">
      <div class="relative">
        <button @click="showSpeedMenu = !showSpeedMenu"
          class="px-2 py-1 rounded text-xs font-medium border hover:border-primary-500 transition-colors"
          style="color: var(--text-secondary); border-color: var(--border-color)">
          {{ player.playbackRate.value }}x
        </button>
        <div v-if="showSpeedMenu" class="absolute bottom-full right-0 mb-1 py-1 rounded-lg shadow-lg border z-50" style="background-color: var(--bg-card); border-color: var(--border-color)">
          <button v-for="speed in speeds" :key="speed"
            @click="player.setRate(speed); showSpeedMenu = false"
            class="block w-full px-3 py-1 text-xs text-left hover:bg-primary-50 dark:hover:bg-primary-900 transition-colors"
            :style="{ color: speed === player.playbackRate.value ? 'var(--color-primary-500)' : 'var(--text-secondary)' }">
            {{ speed }}x
          </button>
        </div>
      </div>
    </div>
  </div>
</template>