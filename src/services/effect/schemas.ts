import { Schema } from "effect"

export const AssociationSchema = Schema.Struct({
  id: Schema.Number,
  name: Schema.String,
})

export const SeasonSchema = Schema.Struct({
  id: Schema.Number,
  association_id: Schema.Number,
  name: Schema.String,
})

export const MatchSchema = Schema.Struct({
  id: Schema.Number,
  season_id: Schema.Number,
  name: Schema.String,
  team_home: Schema.optional(Schema.NullOr(Schema.String)),
  team_away: Schema.optional(Schema.NullOr(Schema.String)),
  has_video: Schema.optional(Schema.Boolean),
  match_date: Schema.optional(Schema.NullOr(Schema.String)),
})

export const ScoutFileSchema = Schema.Struct({
  id: Schema.Number,
  match_id: Schema.Number,
  original_name: Schema.String,
  stored_path: Schema.String,
  source_path: Schema.NullOr(Schema.String),
  file_size: Schema.Number,
  source_format: Schema.String,
})

export const ImportedScoutFileSchema = Schema.Struct({
  association: AssociationSchema,
  season: SeasonSchema,
  match_record: MatchSchema,
  scout_file: ScoutFileSchema,
})

export const ScoutPlayRowSchema = Schema.Struct({
  row_id: Schema.Number,
  raw_line: Schema.String,
  code: Schema.String,
  set_number: Schema.NullOr(Schema.Number),
  score: Schema.NullOr(Schema.String),
  video_time_raw: Schema.NullOr(Schema.String),
  video_time_seconds: Schema.NullOr(Schema.Number),
  match_id: Schema.optional(Schema.NullOr(Schema.Number)),
  match_name: Schema.optional(Schema.NullOr(Schema.String)),
})

export const VideoPathEntrySchema = Schema.Struct({
  match_id: Schema.Number,
  video_path: Schema.NullOr(Schema.String),
})

export const BatchImportFailureSchema = Schema.Struct({
  source_path: Schema.String,
  reason: Schema.String,
})

export const BatchImportResultSchema = Schema.Struct({
  imported: Schema.Array(ImportedScoutFileSchema),
  failed: Schema.Array(BatchImportFailureSchema),
})

export const ExportMontageVideoResultSchema = Schema.Struct({
  output_path: Schema.String,
  clips_exported: Schema.Number,
})
