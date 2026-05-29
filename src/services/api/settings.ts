import { invoke } from "@tauri-apps/api/core"
import type { FileType } from "../../types/database"

export function initApp(): Promise<void> {
  return invoke("init_app")
}

export function getFileType(): Promise<string> {
  return invoke("get_file_type")
}

export function setFileType(fileType: FileType): Promise<void> {
  return invoke("set_file_type", { fileType })
}

export function getAutoSeason(): Promise<boolean> {
  return invoke("get_auto_season")
}

export function setAutoSeason(value: boolean): Promise<void> {
  return invoke("set_auto_season", { value })
}
