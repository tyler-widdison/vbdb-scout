import { ref } from "vue";

export type ExplorerHotkeyAction =
  | "newSeason"
  | "newAssociation"
  | "videoPlayPause"
  | "videoSeekForward"
  | "videoSeekBackward"
  | "videoSeekForwardOneMinute"
  | "nextPlay"
  | "previousPlay"
  | "togglePlaySelection";

export type HotkeyBinding = {
  key: string;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
  meta: boolean;
};

const STORAGE_KEY = "vbdb-scout.explorerHotkeys";

const defaults: Record<ExplorerHotkeyAction, HotkeyBinding | null> = {
  newSeason: { key: "N", ctrl: false, shift: false, alt: false, meta: false },
  newAssociation: {
    key: "A",
    ctrl: false,
    shift: true,
    alt: false,
    meta: false,
  },
  videoPlayPause: {
    key: " ",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
  videoSeekForward: {
    key: "ArrowRight",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
  videoSeekBackward: {
    key: "ArrowLeft",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
  videoSeekForwardOneMinute: null,
  nextPlay: {
    key: "ArrowDown",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
  previousPlay: {
    key: "ArrowUp",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
  togglePlaySelection: {
    key: "Enter",
    ctrl: false,
    shift: false,
    alt: false,
    meta: false,
  },
};

const hotkeys =
  ref<Record<ExplorerHotkeyAction, HotkeyBinding | null>>(loadHotkeys());

function loadHotkeys(): Record<ExplorerHotkeyAction, HotkeyBinding | null> {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return { ...defaults };

  try {
    return { ...defaults, ...JSON.parse(stored) };
  } catch {
    return { ...defaults };
  }
}

function saveHotkeys() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(hotkeys.value));
}

export function formatHotkey(binding: HotkeyBinding | null): string {
  if (!binding) return "Blank";

  const parts = [];
  if (binding.ctrl) parts.push("Ctrl");
  if (binding.meta) parts.push("Meta");
  if (binding.alt) parts.push("Alt");
  if (binding.shift) parts.push("Shift");
  const keyLabel =
    binding.key === " "
      ? "Space"
      : binding.key.length === 1
        ? binding.key.toUpperCase()
        : binding.key;
  parts.push(keyLabel);
  return parts.join(" + ");
}

export function eventToHotkey(e: KeyboardEvent): HotkeyBinding | null {
  if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return null;

  return {
    key: e.key.length === 1 ? e.key.toUpperCase() : e.key,
    ctrl: e.ctrlKey,
    shift: e.shiftKey,
    alt: e.altKey,
    meta: e.metaKey,
  };
}

export function matchesHotkey(
  e: KeyboardEvent,
  binding: HotkeyBinding | null,
): boolean {
  if (!binding) return false;

  const eventKey = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return (
    eventKey === binding.key &&
    e.ctrlKey === binding.ctrl &&
    e.shiftKey === binding.shift &&
    e.altKey === binding.alt &&
    e.metaKey === binding.meta
  );
}

export function useExplorerHotkeys() {
  function setHotkey(
    action: ExplorerHotkeyAction,
    binding: HotkeyBinding | null,
  ) {
    hotkeys.value = { ...hotkeys.value, [action]: binding };
    saveHotkeys();
  }

  function resetHotkey(action: ExplorerHotkeyAction) {
    setHotkey(action, defaults[action]);
  }

  return { hotkeys, setHotkey, resetHotkey };
}
