import type { Theme } from "./types"

export function themeToCssVars(theme: Theme): Record<string, string> {
  const c = theme.colors

  return {
    "--bg": c.bg,
    "--fg": c.fg,
    "--surface": c.surface,
    "--border": c.border,
    "--muted": c.muted,
    "--accent": c.accent,
    "--accent-contrast": c.accentContrast,
    "--glow": c.glow,
    "--gradient-a": c.gradientA,
    "--gradient-b": c.gradientB,
    "--surface-soft": `color-mix(in srgb, ${c.surface} 58%, ${c.bg})`,
    "--border-soft": `color-mix(in srgb, ${c.border} 70%, transparent)`,
    "--text-muted": `color-mix(in srgb, ${c.fg} 62%, ${c.bg})`,
    "--accent-soft": `color-mix(in srgb, ${c.accent} 18%, transparent)`,
    "--accent-border": `color-mix(in srgb, ${c.accent} 42%, ${c.border})`,
    "--shadow-lg": theme.isDark
      ? "0 24px 70px rgba(0, 0, 0, 0.46)"
      : "0 24px 70px rgba(0, 0, 0, 0.18)",
    "--shadow-sm": theme.isDark
      ? "0 10px 28px rgba(0, 0, 0, 0.22)"
      : "0 10px 28px rgba(0, 0, 0, 0.10)",
    "--grain-opacity": theme.isDark ? "0.04" : "0.025",
    "--color-scheme": theme.isDark ? "dark" : "light",
  }
}
