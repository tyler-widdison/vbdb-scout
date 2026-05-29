export interface Association {
  id: number
  name: string
}

export interface Season {
  id: number
  association_id: number
  name: string
}

export interface Match {
  id: number
  season_id: number
  name: string
  team_home?: string | null
  team_away?: string | null
  has_video?: boolean
  match_date?: string | null
}

export interface ScoutFile {
  id: number
  match_id: number
  original_name: string
  stored_path: string
  source_path: string | null
  file_size: number
  source_format: string
}

export interface ImportedScoutFile {
  association: Association
  season: Season
  match_record: Match
  scout_file: ScoutFile
}

export interface ScoutPlayRow {
  row_id: number
  raw_line: string
  code: string
  set_number: number | null
  score: string | null
  video_time_raw: string | null
  video_time_seconds: number | null
  match_id?: number | null
  match_name?: string | null
}

export interface VideoPathEntry {
  match_id: number
  video_path: string | null
}

export type FileType = "json" | "txt" | "csv" | "dvw"
