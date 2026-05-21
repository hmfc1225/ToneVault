<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useAudioPlayer } from '@/composables/useAudioPlayer'
import { getBook, getBookTracks, getBookAuthors } from '@/api/books'
import { createBookmark, listBookmarks } from '@/api/bookmarks'
import { listCollections, addBookToCollection } from '@/api/collections'
import { getUserPositions } from '@/api/positions'
import type { Book, Track, AuthorWithRole, Bookmark, Collection, PlaybackPosition } from '@/types'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()
const player = useAudioPlayer()

const book = ref<Book | null>(null)
const tracks = ref<Track[]>([])
const authors = ref<AuthorWithRole[]>([])
const bookmarks = ref<Bookmark[]>([])
const collections = ref<Collection[]>([])
const position = ref<PlaybackPosition | null>(null)
const loading = ref(true)
const showBookmarkDialog = ref(false)
const bookmarkTitle = ref('')
const bookmarkNote = ref('')
const selectedCollection = ref('')

const bookId = computed(() => route.params.id as string)

function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = Math.floor(secs % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatTotalDuration(secs: number): string {
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  if (h > 0) return `${h}小时${m}分钟`
  return `${m}分钟`
}

async function loadData() {
  loading.value = true
  try {
    const [b, t, a] = await Promise.all([
      getBook(bookId.value),
      getBookTracks(bookId.value),
      getBookAuthors(bookId.value),
    ])
    book.value = b
    tracks.value = t.sort((x, y) => x.track_number - y.track_number)
    authors.value = a

    if (auth.user) {
      const [bm, colls, positions] = await Promise.all([
        listBookmarks(auth.user.id, bookId.value).catch(() => []),
        listCollections(auth.user.id).catch(() => []),
        getUserPositions(auth.user.id).catch(() => []),
      ])
      bookmarks.value = bm
      collections.value = colls
      position.value = positions.find(p => p.book_id === bookId.value) || null
    }
  } catch { /* ignore */ } finally {
    loading.value = false
  }
}

function playFromTrack(track: Track, startTime?: number) {
  if (!book.value) return
  player.play(book.value, tracks.value, track, startTime)
}

function playFromBeginning() {
  if (!book.value || tracks.value.length === 0) return
  player.play(book.value, tracks.value)
}

function continueListening() {
  if (!book.value || !position.value) return
  const startTrack = tracks.value.find(t => t.id === position.value!.track_id) || tracks.value[0]
  if (!startTrack) return
  player.play(book.value, tracks.value, startTrack, Math.floor(position.value.position_secs))
}

async function addBookmark() {
  if (!auth.user || !book.value) return
  try {
    const track = player.currentTrack.value || tracks.value[0]
    if (!track) return
    await createBookmark(auth.user.id, {
      book_id: book.value.id,
      track_id: track.id,
      title: bookmarkTitle.value || `第${player.currentTrack.value?.track_number || 1}章标记`,
      position_secs: Math.floor(player.currentTime.value),
      note: bookmarkNote.value || undefined,
    })
    bookmarkTitle.value = ''
    bookmarkNote.value = ''
    showBookmarkDialog.value = false
    if (auth.user) {
      bookmarks.value = await listBookmarks(auth.user.id, bookId.value)
    }
  } catch { /* ignore */ }
}

async function addToCollection(collectionId: string) {
  if (!collectionId || !book.value) return
  try {
    await addBookToCollection(collectionId, book.value.id)
    selectedCollection.value = ''
  } catch { /* ignore */ }
}

onMounted(loadData)
</script>

<template>
  <div v-if="loading" class="text-center py-12" style="color: var(--text-secondary)">加载中...</div>

  <div v-else-if="book" class="space-y-6">
    <button
      class="text-sm font-medium flex items-center gap-1"
      style="color: var(--text-secondary)"
      @click="router.push('/books')"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" /></svg>
      返回列表
    </button>

    <div class="flex gap-6">
      <div class="w-40 h-56 rounded-lg flex-shrink-0 flex items-center justify-center" style="background-color: var(--bg-secondary)">
        <svg class="w-16 h-16" style="color: var(--text-secondary)" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
      </div>
      <div class="flex-1">
        <h1 class="text-2xl font-bold" style="color: var(--text-primary)">{{ book.title }}</h1>
        <p class="mt-1" style="color: var(--text-secondary)">
          {{ authors.map(a => a.author.name).join(', ') || '未知作者' }}
        </p>
        <div class="mt-2 flex flex-wrap gap-3 text-sm" style="color: var(--text-secondary)">
          <span v-if="book.duration_secs">{{ formatTotalDuration(book.duration_secs) }}</span>
          <span v-if="book.track_count">{{ book.track_count }} 个章节</span>
          <span v-if="book.file_size">{{ (book.file_size / 1024 / 1024).toFixed(1) }} MB</span>
        </div>
        <div class="mt-4 flex flex-wrap gap-2">
          <button
            v-if="position && position.position_secs > 0 && !position.is_finished"
            class="px-4 py-2 rounded-lg text-white font-medium text-sm"
            style="background-color: var(--color-primary)"
            @click="continueListening"
          >
            继续收听
          </button>
          <button
            class="px-4 py-2 rounded-lg font-medium text-sm border"
            style="border-color: var(--border-color); color: var(--text-primary)"
            @click="playFromBeginning"
          >
            从头播放
          </button>
          <button
            v-if="player.currentBook.value?.id === book.id"
            class="px-4 py-2 rounded-lg font-medium text-sm border"
            style="border-color: var(--border-color); color: var(--text-primary)"
            @click="showBookmarkDialog = !showBookmarkDialog"
          >
            添加书签
          </button>
          <select
            v-if="collections.length > 0"
            v-model="selectedCollection"
            class="px-3 py-2 rounded-lg text-sm border outline-none"
            style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)"
            @change="addToCollection(selectedCollection)"
          >
            <option value="">加入收藏夹...</option>
            <option v-for="c in collections" :key="c.id" :value="c.id">{{ c.name }}</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="showBookmarkDialog" class="p-4 rounded-lg space-y-3" style="background-color: var(--bg-card)">
      <input
        v-model="bookmarkTitle"
        type="text"
        placeholder="书签标题"
        class="w-full px-3 py-2 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)"
      />
      <textarea
        v-model="bookmarkNote"
        placeholder="备注（可选）"
        rows="2"
        class="w-full px-3 py-2 rounded-lg border outline-none focus:ring-2 focus:ring-primary-500"
        style="background-color: var(--bg-secondary); border-color: var(--border-color); color: var(--text-primary)"
      />
      <button
        class="px-4 py-2 rounded-lg text-white font-medium text-sm"
        style="background-color: var(--color-primary)"
        @click="addBookmark"
      >
        保存书签
      </button>
    </div>

    <div v-if="bookmarks.length > 0" class="space-y-2">
      <h2 class="text-lg font-semibold" style="color: var(--text-primary)">书签</h2>
      <div
        v-for="bm in bookmarks"
        :key="bm.id"
        class="p-3 rounded-lg flex items-center justify-between"
        style="background-color: var(--bg-card)"
      >
        <div>
          <span class="font-medium text-sm" style="color: var(--text-primary)">{{ bm.title }}</span>
          <span class="text-xs ml-2" style="color: var(--text-secondary)">{{ formatDuration(bm.position_secs) }}</span>
          <p v-if="bm.note" class="text-xs mt-0.5" style="color: var(--text-secondary)">{{ bm.note }}</p>
        </div>
        <button
          class="text-xs px-2 py-1 rounded"
          style="color: var(--text-secondary)"
          @click="playFromTrack(tracks.find(t => t.id === bm.track_id)!, Math.floor(bm.position_secs))"
        >
          播放
        </button>
      </div>
    </div>

    <div>
      <h2 class="text-lg font-semibold mb-3" style="color: var(--text-primary)">章节列表</h2>
      <div class="space-y-1">
        <div
          v-for="track in tracks"
          :key="track.id"
          class="p-3 rounded-lg flex items-center justify-between cursor-pointer hover:opacity-80"
          :style="{ backgroundColor: player.currentTrack.value?.id === track.id ? 'var(--bg-card)' : 'transparent' }"
          @click="playFromTrack(track)"
        >
          <div class="flex items-center gap-3">
            <span class="text-sm w-6 text-right" style="color: var(--text-secondary)">{{ track.track_number }}</span>
            <span class="text-sm" style="color: var(--text-primary)">{{ track.title || `第${track.track_number}章` }}</span>
          </div>
          <span class="text-sm" style="color: var(--text-secondary)">{{ formatDuration(track.duration_secs) }}</span>
        </div>
      </div>
    </div>
  </div>

  <div v-else class="text-center py-12" style="color: var(--text-secondary)">有声书未找到</div>
</template>
