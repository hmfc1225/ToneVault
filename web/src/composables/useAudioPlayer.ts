import { ref, watch, onMounted, onUnmounted } from 'vue'
import type { Book, Track } from '@/types'
import { getTrackStreamUrl } from '@/api/books'
import * as positionsApi from '@/api/positions'
import { useAuthStore } from '@/stores/auth'

const audio = new Audio()
const currentBook = ref<Book | null>(null)
const currentTrack = ref<Track | null>(null)
const queue = ref<Track[]>([])
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const playbackRate = ref(1)
const isLoading = ref(false)

let positionSaveInterval: ReturnType<typeof setInterval> | null = null
let lastSavedPosition = 0

function loadTrack(track: Track, startPosition?: number) {
  currentTrack.value = track
  audio.src = getTrackStreamUrl(track.id)
  audio.playbackRate = playbackRate.value

  if (startPosition !== undefined && startPosition > 0) {
    audio.addEventListener('loadedmetadata', function onLoaded() {
      audio.currentTime = startPosition
      audio.removeEventListener('loadedmetadata', onLoaded)
    })
  }

  audio.play().catch(() => {})
}

function savePosition() {
  if (!currentBook.value || !currentTrack.value) return
  const pos = Math.floor(audio.currentTime)
  if (pos === lastSavedPosition) return
  lastSavedPosition = pos

  const auth = useAuthStore()
  if (!auth.user) return

  positionsApi.upsertPosition(auth.user.id, {
    book_id: currentBook.value.id,
    track_id: currentTrack.value.id,
    position_secs: pos,
    percentage: duration.value > 0 ? pos / duration.value : 0,
    duration_secs: Math.floor(duration.value),
  }).catch(() => {})
}

function playNext() {
  if (!currentTrack.value || queue.value.length === 0) return
  const idx = queue.value.findIndex(t => t.id === currentTrack.value!.id)
  if (idx < queue.value.length - 1) {
    savePosition()
    loadTrack(queue.value[idx + 1])
  } else {
    audio.pause()
    isPlaying.value = false
    savePosition()
  }
}

function stopPositionSave() {
  if (positionSaveInterval) {
    clearInterval(positionSaveInterval)
    positionSaveInterval = null
  }
}

function startPositionSave() {
  stopPositionSave()
  positionSaveInterval = setInterval(() => {
    savePosition()
  }, 15000)
}

audio.addEventListener('timeupdate', () => {
  currentTime.value = audio.currentTime
})

audio.addEventListener('durationchange', () => {
  duration.value = audio.duration || 0
})

audio.addEventListener('ended', () => {
  playNext()
})

audio.addEventListener('play', () => {
  isPlaying.value = true
})

audio.addEventListener('pause', () => {
  isPlaying.value = false
})

audio.addEventListener('waiting', () => {
  isLoading.value = true
})

audio.addEventListener('canplay', () => {
  isLoading.value = false
})

watch(playbackRate, (rate) => {
  audio.playbackRate = rate
})

export function useAudioPlayer() {
  function play(book: Book, tracks: Track[], startTrack?: Track, startPosition?: number) {
    currentBook.value = book
    queue.value = [...tracks].sort((a, b) => a.track_number - b.track_number)

    const track = startTrack || queue.value[0]
    if (!track) return

    loadTrack(track, startPosition)
    startPositionSave()
  }

  function togglePlay() {
    if (isPlaying.value) {
      audio.pause()
    } else {
      audio.play().catch(() => {})
    }
  }

  function pause() {
    audio.pause()
  }

  function resume() {
    audio.play().catch(() => {})
  }

  function seek(time: number) {
    audio.currentTime = time
  }

  function seekByPercent(percent: number) {
    if (duration.value > 0) {
      audio.currentTime = duration.value * percent
    }
  }

  function playPrev() {
    if (!currentTrack.value || queue.value.length === 0) return
    if (audio.currentTime > 3) {
      audio.currentTime = 0
      return
    }
    const idx = queue.value.findIndex(t => t.id === currentTrack.value!.id)
    if (idx > 0) {
      savePosition()
      loadTrack(queue.value[idx - 1])
    }
  }

  function setRate(rate: number) {
    playbackRate.value = rate
  }

  function stop() {
    audio.pause()
    audio.src = ''
    currentBook.value = null
    currentTrack.value = null
    queue.value = []
    isPlaying.value = false
    currentTime.value = 0
    duration.value = 0
    stopPositionSave()
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return
    if (e.code === 'Space' && currentBook.value) {
      e.preventDefault()
      togglePlay()
    } else if (e.code === 'ArrowLeft' && currentBook.value) {
      e.preventDefault()
      seek(Math.max(0, audio.currentTime - 10))
    } else if (e.code === 'ArrowRight' && currentBook.value) {
      e.preventDefault()
      seek(Math.min(duration.value, audio.currentTime + 10))
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return {
    currentBook,
    currentTrack,
    queue,
    isPlaying,
    currentTime,
    duration,
    playbackRate,
    isLoading,
    play,
    togglePlay,
    pause,
    resume,
    seek,
    seekByPercent,
    playNext,
    playPrev,
    setRate,
    stop,
    savePosition,
  }
}
