# VBDB Scout

Tauri + Vue desktop app for (DataVolley `.dvw` files).

## Stack

- **Frontend:** Vue 3 + TypeScript + Vite
- **Backend:** Rust (Tauri v2)
- **State:** Vue composables with shared refs
- **Styling:** CSS custom properties via theme system (`src/themes/`)
- **Package manager:** bun

## Commands

- `bun run dev` — Vite dev server (port 1420 for Tauri)
- `bun run build` — typecheck + build
- `bun run typecheck` — typecheck only

<!-- effect-solutions:start -->

## Effect Best Practices

**IMPORTANT:** Always consult effect-solutions before writing Effect code.

1. Run `effect-solutions list` to see available guides
2. Run `effect-solutions show <topic>...` for relevant patterns (supports multiple topics)
3. Search `~/.local/share/effect-solutions/effect` for real implementations

Topics: quick-start, project-setup, tsconfig, basics, services-and-layers, data-modeling, error-handling, config, testing, cli.

Never guess at Effect patterns - check the guide first.

<!-- effect-solutions:end -->

## Architecture

- `src/views/` — route-level page components
- `src/components/` — reusable UI components
- `src/components/common/` — shared primitives (dialogs, etc.)
- `src/components/match/` — match-specific components
- `src/components/tree/` — file explorer tree
- `src/composables/` — Vue composition functions (shared state via module-level refs)
- `src/services/api/` — Tauri IPC wrappers (`invoke()` calls)
- `src/themes/` — theme definitions and CSS variable generation
- `src/types/` — TypeScript interfaces
- `src/constants/` — static data (DataVolley codes, etc.)
- `src-tauri/` — Rust backend

## Theme System

Themes define CSS custom properties via `themeToCssVars()`. Key variables:

- `--bg`, `--fg`, `--surface`, `--border`, `--muted`, `--accent`
- Derived: `--surface-soft`, `--border-soft`, `--text-muted`, `--accent-soft`, `--accent-border`
- Use `color-mix()` for transparency variants

## Conventions

- Scoped styles in SFCs
- `invoke()` for all backend calls via Tauri
- Fonts: Plus Jakarta Sans (UI) + Cascadia Mono (headings)
