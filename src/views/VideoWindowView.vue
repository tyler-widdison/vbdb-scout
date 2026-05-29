<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue"

type VideoMessage =
  | { type: "load"; src: string }
  | { type: "seek"; time: number; autoplay?: boolean }

const videoEl = ref<HTMLVideoElement | null>(null)
const status = ref("Waiting for video...")
const channel = new BroadcastChannel("vbdb-scout-video-control")

function onMessage(event: MessageEvent<VideoMessage>) {
  const data = event.data
  if (!videoEl.value || !data) return

  if (data.type === "load" && data.src) {
    if (videoEl.value.src !== data.src) {
      videoEl.value.src = data.src
      videoEl.value.load()
    }
    status.value = "Video loaded"
    return
  }

  if (data.type === "seek" && typeof data.time === "number") {
    videoEl.value.currentTime = Math.max(0, data.time)
    if (data.autoplay) {
      videoEl.value.play().catch(() => {})
    }
  }
}

onMounted(() => {
  channel.addEventListener("message", onMessage as EventListener)
})

onBeforeUnmount(() => {
  channel.removeEventListener("message", onMessage as EventListener)
  channel.close()
})
</script>

<template>
  <div class="video-window">
    <div class="status">{{ status }}</div>
    <video ref="videoEl" controls preload="metadata"></video>
  </div>
</template>

<style scoped>
.video-window {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #111;
}

.status {
  color: #ddd;
  font-size: 12px;
  padding: 8px 10px;
  background: #1b1f2b;
  border-bottom: 1px solid #2d3242;
}

video {
  width: 100%;
  height: calc(100vh - 34px);
  background: #000;
}
</style>
