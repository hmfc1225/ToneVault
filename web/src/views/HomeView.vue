<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getUserPositions } from '@/api/positions'
import { getBook, getBookTracks } from '@/api/books'
import type { Book, Track, PlaybackPosition } from '@/types'

const router = useRouter()
const auth = useAuthStore()

const recentBooks = ref<{ book: Book; position: PlaybackPosition; tracks: Track[] }[]>([])
const loading = ref(true)

const hasRecent = computed(() => recentBooks.value.length > 0)

function formatProgress(position: PlaybackPosition): string {
  const mins = Math.floor(position.position_secs / 60)
  const secs = Math.floor(position.position_secs % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

function progressPercent(position: PlaybackPosition): number {
  return Math.round(position.percentage * 100)
}

function continueListening(book: Book, tracks: Track[], position: PlaybackPosition) {
  const startTrack = tracks.find(t => t.id === position.track_id) || tracks[0]
  if (!startTrack) return
  router.push({
    path: `/books/${book.id}`,
    query: { play: 'true', track: startTrack.id, time: String(Math.floor(position.position_secs)) }
  })
}

onMounted(async () => {
  if (!auth.user) { loading.value = false; return }
  try {
    const positions = await getUserPositions(auth.user.id)
    const unfinished = positions
      .filter(p => !p.is_finished && p.position_secs > 0)
      .sort((a, b) => b.position_secs - a.position_secs)
      .slice(0, 6)

    const results = await Promise.all(
      unfinished.map(async (pos) => {
        try {
          const [book, tracks] = await Promise.all([
            getBook(pos.book_id),
            getBookTracks(pos.book_id),
          ])
          return { book, position: pos, tracks }
        } catch { return null }
      })
    )
    recentBooks.value = results.filter((r): r is { book: Book; position: PlaybackPosition; tracks: Track[] } => r !== null)
  } catch {
    // ignore
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="space-y-8">
    <div>
      <h1 class="text-2xl font-bold" style="color: var(--text-primary)">首页</h1>
      <p class="mt-1" style="color: var(--text-secondary)">欢迎回来，{{ auth.user?.display_name || auth.user?.username }}</p>
    </div>

    <section v-if="loading">
      <p style="color: var(--text-secondary)">加载中...</p>
    </section>

    <section v-else-if="hasRecent">
      <h2 class="text-lg font-semibold mb-4" style="color: var(--text-primary)">继续收听</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        <div
          v-for="item in recentBooks"
          :key="item.book.id"
          class="p-4 rounded-lg cursor-pointer transition-shadow hover:shadow-md"
          style="background-color: var(--bg-card)"
          @click="continueListening(item.book, item.tracks, item.position)"
        >
          <div class="flex gap-3">
            <div class="w-16 h-20 rounded flex-shrink-0 flex items-center justify-center" style="background-color: var(--bg-secondary)">
              <svg class="w-6 h-6" style="color: var(--text-secondary)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="font-medium truncate" style="color: var(--text-primary)">{{ item.book.title }}</h3>
              <p class="text-sm truncate" style="color: var(--text-secondary)">
                {{ item.book.authors?.map(a => a.name).join(', ') || '未知作者' }}
              </p>
              <div class="mt-2">
                <div class="w-full h-1.5 rounded-full" style="background-color: var(--bg-secondary)">
                  <div
                    class="h-full rounded-full transition-all"
                    style="background-color: var(--color-primary)"
                    :style="{ width: progressPercent(item.position) + '%' }"
                  />
                </div>
                <div class="flex justify-between mt-1 text-xs" style="color: var(--text-secondary)">
                  <span>{{ formatProgress(item.position) }}</span>
                  <span>{{ progressPercent(item.position) }}%</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section v-else class="text-center py-12">
      <svg class="w-16 h-16 mx-auto mb-4" style="color: var(--text-secondary)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
      </svg>
      <h3 class="text-lg font-medium" style="color: var(--text-primary)">暂无收听记录</h3>
      <p class="mt-1" style="color: var(--text-secondary)">浏览书库，开始你的有声书之旅</p>
      <button
        class="mt-4 px-4 py-2 rounded-lg text-white font-medium"
        style="background-color: var(--color-primary)"
        @click="router.push('/books')"
      >
        浏览书库
      </button>
    </section>
  </div>
</template>
