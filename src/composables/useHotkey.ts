import { onMounted, onBeforeUnmount } from "vue"

type KeyCombo = {
  key: string
  ctrl?: boolean
  meta?: boolean
  shift?: boolean
  alt?: boolean
}

export function useHotkey(combo: KeyCombo, handler: (e: KeyboardEvent) => void) {
  function onKeydown(e: KeyboardEvent) {
    if (combo.ctrl && !e.ctrlKey && !e.metaKey) return
    if (combo.meta && !e.metaKey) return
    if (combo.shift && !e.shiftKey) return
    if (combo.alt && !e.altKey) return
    if (e.key !== combo.key) return
    e.preventDefault()
    handler(e)
  }

  onMounted(() => document.addEventListener("keydown", onKeydown))
  onBeforeUnmount(() => document.removeEventListener("keydown", onKeydown))
}
