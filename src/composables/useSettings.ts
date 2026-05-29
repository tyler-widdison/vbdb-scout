import { ref } from "vue"

export type SettingsTab = "general" | "settings" | "theme" | "hotkeys"

const open = ref(false)
const activeTab = ref<SettingsTab>("general")

function openSettings(tab?: SettingsTab) {
  open.value = true
  if (tab) activeTab.value = tab
}

function closeSettings() {
  open.value = false
}

function toggleSettings() {
  if (open.value) closeSettings()
  else openSettings("general")
}

export function useSettings() {
  return { open, activeTab, openSettings, closeSettings, toggleSettings }
}
