import { Schema } from "effect"
import type { FileType } from "../../types/database"
import { tauri, tauriVoid } from "../effect/tauri"

export function initApp(): Promise<void> {
  return tauriVoid("init_app", {})
}

export function getFileType(): Promise<string> {
  return tauri("get_file_type", {}, Schema.String)
}

export function setFileType(fileType: FileType): Promise<void> {
  return tauriVoid("set_file_type", { fileType })
}

export function getAutoSeason(): Promise<boolean> {
  return tauri("get_auto_season", {}, Schema.Boolean)
}

export function setAutoSeason(value: boolean): Promise<void> {
  return tauriVoid("set_auto_season", { value })
}

export function getStoredScoutFilesPath(): Promise<string> {
  return tauri("get_stored_scout_files_path", {}, Schema.String)
}
