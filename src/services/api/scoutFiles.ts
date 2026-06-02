import { Schema } from "effect"
import type { ImportedScoutFile, ScoutPlayRow, VideoPathEntry } from "../../types/database"
import {
  BatchImportResultSchema,
  ExportMontageVideoResultSchema,
  ImportedScoutFileSchema,
  ScoutPlayRowSchema,
  VideoPathEntrySchema,
} from "../effect/schemas"
import { tauri, tauriVoid } from "../effect/tauri"

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

export type ScoutCodeChange = {
  match_id: number
  row_id: number
  code: string
}

export function importScoutFile(input: ImportScoutFileInput): Promise<ImportedScoutFile> {
  return tauri("import_scout_file", input as unknown as Record<string, unknown>, ImportedScoutFileSchema) as Promise<ImportedScoutFile>
}

export function importScoutFiles(
  sourcePaths: string[],
  associationName: string,
  fallbackSeasonName: string,
  autoSeason: boolean,
): Promise<BatchImportResult> {
  return tauri("import_scout_files", { sourcePaths, associationName, fallbackSeasonName, autoSeason }, BatchImportResultSchema) as Promise<BatchImportResult>
}

export function getScoutLines(matchId: number): Promise<string[]> {
  return tauri("get_scout_lines", { matchId }, Schema.Array(Schema.String)) as Promise<string[]>
}

export function getScoutRows(matchId: number): Promise<ScoutPlayRow[]> {
  return tauri("get_scout_rows", { matchId }, Schema.Array(ScoutPlayRowSchema)) as Promise<ScoutPlayRow[]>
}

export function getScoutVideoPath(matchId: number): Promise<string | null> {
  return tauri("get_scout_video_path", { matchId }, Schema.NullOr(Schema.String))
}

export function getScoutRowsMulti(matchIds: number[]): Promise<ScoutPlayRow[]> {
  return tauri("get_scout_rows_multi", { matchIds }, Schema.Array(ScoutPlayRowSchema)) as Promise<ScoutPlayRow[]>
}

export function getScoutRowsMultiFiltered(
  matchIds: number[],
  filters: ScoutFilterRow[],
): Promise<ScoutPlayRow[]> {
  return tauri("get_scout_rows_multi_filtered", { matchIds, filters: filters as unknown as Record<string, unknown>[] }, Schema.Array(ScoutPlayRowSchema)) as Promise<ScoutPlayRow[]>
}

export function getScoutVideoPathsMulti(matchIds: number[]): Promise<VideoPathEntry[]> {
  return tauri("get_scout_video_paths_multi", { matchIds }, Schema.Array(VideoPathEntrySchema)) as Promise<VideoPathEntry[]>
}

export function exportMontageVideo(
  input: ExportMontageVideoInput,
): Promise<ExportMontageVideoResult> {
  return tauri("export_montage_video", input as unknown as Record<string, unknown>, ExportMontageVideoResultSchema) as Promise<ExportMontageVideoResult>
}

export function updateScoutCodes(changes: ScoutCodeChange[]): Promise<void> {
  return tauriVoid("update_scout_codes", { changes })
}
