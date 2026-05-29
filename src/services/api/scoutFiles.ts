import { invoke } from "@tauri-apps/api/core"
import type { ImportedScoutFile, ScoutPlayRow, VideoPathEntry } from "../../types/database"

export type ScoutFilterRow = {
  relation?: string
  condition?: "contains" | "not_contains"
  team: string
  number: string
  skill: string
  subType: string
  grade: string
  combo: string
  startZone: string
  endZone: string
  skillType: string
  players: string
}

export type BatchImportFailure = {
  source_path: string
  reason: string
}

export type BatchImportResult = {
  imported: ImportedScoutFile[]
  failed: BatchImportFailure[]
}

export type ImportScoutFileInput = {
  sourcePath: string
  associationName: string
  seasonName: string
  matchName: string
}

export type MontageClipInput = {
  row_id: number
  match_id: number | null
  match_name: string | null
  video_path: string
  start_time: number
  end_time: number
  code: string
  video_time_seconds: number
}

export type ExportMontageVideoInput = {
  outputPath: string
  clips: MontageClipInput[]
}

export type ExportMontageVideoResult = {
  output_path: string
  clips_exported: number
}

export function importScoutFile(input: ImportScoutFileInput): Promise<ImportedScoutFile> {
  return invoke("import_scout_file", input)
}

export function importScoutFiles(
  sourcePaths: string[],
  associationName: string,
  fallbackSeasonName: string,
  autoSeason: boolean,
): Promise<BatchImportResult> {
  return invoke("import_scout_files", { sourcePaths, associationName, fallbackSeasonName, autoSeason })
}

export function getScoutLines(matchId: number): Promise<string[]> {
  return invoke("get_scout_lines", { matchId })
}

export function getScoutRows(matchId: number): Promise<ScoutPlayRow[]> {
  return invoke("get_scout_rows", { matchId })
}

export function getScoutVideoPath(matchId: number): Promise<string | null> {
  return invoke("get_scout_video_path", { matchId })
}

export function getScoutRowsMulti(matchIds: number[]): Promise<ScoutPlayRow[]> {
  return invoke("get_scout_rows_multi", { matchIds })
}

export function getScoutRowsMultiFiltered(
  matchIds: number[],
  filters: ScoutFilterRow[],
): Promise<ScoutPlayRow[]> {
  return invoke("get_scout_rows_multi_filtered", { matchIds, filters })
}

export function getScoutVideoPathsMulti(matchIds: number[]): Promise<VideoPathEntry[]> {
  return invoke("get_scout_video_paths_multi", { matchIds })
}

export function exportMontageVideo(
  input: ExportMontageVideoInput,
): Promise<ExportMontageVideoResult> {
  return invoke("export_montage_video", input)
}
