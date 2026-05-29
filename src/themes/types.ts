export interface ThemeColors {
  bg: string
  fg: string
  surface: string
  border: string
  muted: string
  accent: string
  accentContrast: string
  glow: string
  gradientA: string
  gradientB: string
}

export interface Theme {
  name: string
  colors: ThemeColors
  isDark: boolean
}
