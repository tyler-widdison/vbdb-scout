import { ref, watch, computed } from "vue"
import { themes, DEFAULT_THEME, themeToCssVars } from "../themes"
import type { Theme } from "../themes"

const STORAGE_KEY = "vbdb-scout:theme"

const currentName = ref(load())

function load(): string {
  const stored = getStoredThemeName()
  if (stored) return stored
  return DEFAULT_THEME
}

function getStoredThemeName(): string | null {
  let stored: string | null = null
  try {
    stored = localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
  if (!stored) return null
  if (themes.some((t) => t.name === stored)) return stored
  try {
    localStorage.setItem(STORAGE_KEY, DEFAULT_THEME)
  } catch {
    return null
  }
  return DEFAULT_THEME
}

function apply(name: string) {
  const theme = themes.find((t) => t.name === name) ?? themes.find((t) => t.name === DEFAULT_THEME)!
  const vars = themeToCssVars(theme)
  const root = document.documentElement.style
  for (const [key, value] of Object.entries(vars)) {
    root.setProperty(key, value)
  }
  root.setProperty("color-scheme", vars["--color-scheme"])
}

watch(currentName, (name) => {
  try {
    localStorage.setItem(STORAGE_KEY, name)
  } catch {
    // ignore storage write failures and still apply theme
  }
  apply(name)
}, { immediate: true })

const currentTheme = computed<Theme>(() => themes.find((t) => t.name === currentName.value) ?? themes[0])

export function useTheme() {
  return { currentName, currentTheme, themes }
}
