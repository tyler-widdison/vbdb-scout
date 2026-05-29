<script setup lang="ts">
import { ref, watch } from "vue"

const props = defineProps<{
  source: string
  seekTime: number | null
  clipEndTime: number | null
  playToggleToken: number
  autoPlayOnSeek: boolean
  muteOnAutoplayStart: boolean
}>()

const emit = defineEmits<{
  clipEnded: []
}>()

const videoEl = ref<HTMLVideoElement | null>(null)
const activeSrc = () => props.source
const clipEndHandled = ref(false)

watch(() => props.seekTime, (time) => {
  if (time == null || !videoEl.value) return
  clipEndHandled.value = false
  videoEl.value.currentTime = Math.max(0, time)
  if (props.autoPlayOnSeek) {
    videoEl.value.muted = props.muteOnAutoplayStart
    videoEl.value.play().catch(() => {})
  }
})

watch(() => props.clipEndTime, () => {
  clipEndHandled.value = false
})

watch(() => props.source, () => {
  clipEndHandled.value = false
})

watch(() => props.muteOnAutoplayStart, (muted) => {
  if (!videoEl.value) return
  videoEl.value.muted = muted
}, { immediate: true })

watch(() => props.playToggleToken, () => {
  if (!videoEl.value || !activeSrc()) return
  if (videoEl.value.paused) {
    videoEl.value.play().catch(() => {})
    return
  }
  videoEl.value.pause()
})

function onTimeUpdate() {
  if (!videoEl.value) return
  if (props.clipEndTime == null) return
  if (clipEndHandled.value) return
  if (videoEl.value.currentTime < props.clipEndTime) return
  clipEndHandled.value = true
  videoEl.value.pause()
  emit("clipEnded")
}

function seekBy(seconds: number) {
  if (!videoEl.value || !activeSrc()) return
  const next = Math.max(0, videoEl.value.currentTime + seconds)
  videoEl.value.currentTime = next
  clipEndHandled.value = false
}

defineExpose({ seekBy })

</script>

<template>
  <aside class="video-panel">
    <video v-if="activeSrc()" ref="videoEl" class="video-player" :src="activeSrc()" :muted="muteOnAutoplayStart" controls preload="metadata" @loadedmetadata="videoEl && (videoEl.muted = muteOnAutoplayStart)" @timeupdate="onTimeUpdate"></video>
    <p v-else class="empty">No video loaded.</p>
  </aside>
</template>

<style scoped>
.video-panel {
  width: 100%;
  height: calc(100% - 28px);
  min-width: 0;
  border: 0;
  border-top: 0;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  background: transparent;
}
.video-player { width: 100%; min-width: 0; flex: 1; min-height: 180px; background:#000; border:1px solid var(--border-soft); border-radius:0 0 8px 8px; }
.empty { color: var(--text-muted); font-size: 12px; }
</style>
