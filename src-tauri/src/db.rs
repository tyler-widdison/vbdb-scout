use crate::parsers;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Result, ToSql};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
    #[allow(dead_code)]
    pub app_data_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Association {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Season {
    pub id: i64,
    pub association_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub id: i64,
    pub season_id: i64,
    pub name: String,
    pub team_home: Option<String>,
    pub team_away: Option<String>,
    pub has_video: bool,
    pub match_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoutFile {
    pub id: i64,
    pub match_id: i64,
    pub original_name: String,
    pub stored_path: String,
    pub source_path: Option<String>,
    pub file_size: i64,
    pub source_format: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImportedScoutFile {
    pub association: Association,
    pub season: Season,
    pub match_record: Match,
    pub scout_file: ScoutFile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchImportFailure {
    pub source_path: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchImportResult {
    pub imported: Vec<ImportedScoutFile>,
    pub failed: Vec<BatchImportFailure>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoutPlayRow {
    pub row_id: i64,
    pub raw_line: String,
    pub code: String,
    pub set_number: Option<i64>,
    pub score: Option<String>,
    pub video_time_raw: Option<String>,
    pub video_time_seconds: Option<f64>,
    pub match_id: Option<i64>,
    pub match_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ScoutFilterRow {
    pub relation: Option<String>,
    pub condition: Option<String>,
    pub team: String,
    pub number: String,
    #[serde(rename = "skill")]
    pub skill: String,
    #[serde(rename = "subType")]
    pub sub_type: String,
    pub grade: String,
    pub combo: String,
    #[serde(rename = "startZone")]
    pub start_zone: String,
    #[serde(rename = "endZone")]
    pub end_zone: String,
    #[serde(rename = "skillType")]
    pub skill_type: String,
    pub players: String,
}

#[derive(Debug, Clone, Default)]
struct ParsedCodeFields {
    team: String,
    number: String,
    skill: String,
    sub_type: String,
    grade: String,
    combo: String,
    start_zone: String,
    end_zone: String,
    skill_type: String,
    players: String,
}

const SCOUT_ROW_CACHE_VERSION: i64 = 4;

const SCHEMA: &str = "
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS associations (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY,
    association_id INTEGER NOT NULL REFERENCES associations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_seasons_association_name
    ON seasons(association_id, name);

CREATE TABLE IF NOT EXISTS matches (
    id INTEGER PRIMARY KEY,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_matches_season_name
    ON matches(season_id, name);

CREATE INDEX IF NOT EXISTS idx_matches_season
    ON matches(season_id);

CREATE TABLE IF NOT EXISTS scout_files (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    original_name TEXT NOT NULL,
    stored_path TEXT NOT NULL,
    source_path TEXT,
    file_size INTEGER NOT NULL,
    source_format TEXT NOT NULL,
    has_video INTEGER NOT NULL DEFAULT 0,
    imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_scout_files_match
    ON scout_files(match_id);

CREATE TABLE IF NOT EXISTS scout_play_rows (
    id INTEGER PRIMARY KEY,
    match_id INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    row_id INTEGER NOT NULL,
    raw_line TEXT NOT NULL,
    code TEXT NOT NULL,
    set_number INTEGER,
    score TEXT,
    video_time_raw TEXT,
    video_time_seconds REAL,
    team TEXT,
    number TEXT,
    skill TEXT,
    sub_type TEXT,
    grade TEXT,
    combo TEXT,
    start_zone TEXT,
    end_zone TEXT,
    skill_type TEXT,
    players TEXT,
    rally_id INTEGER,
    cache_version INTEGER NOT NULL DEFAULT 1
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_scout_play_rows_match_row
    ON scout_play_rows(match_id, row_id);

CREATE INDEX IF NOT EXISTS idx_scout_play_rows_match_skill
    ON scout_play_rows(match_id, skill);

CREATE INDEX IF NOT EXISTS idx_scout_play_rows_filter
    ON scout_play_rows(skill, grade, team, number);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

fn seed(db: &Connection) -> Result<()> {
    db.execute(
        "INSERT OR IGNORE INTO settings (key, value) VALUES ('file_type', 'json')",
        [],
    )?;

    db.execute("INSERT OR IGNORE INTO associations (name) VALUES ('VBDB')", [])?;

    Ok(())
}

pub fn init_db(app_data_dir: &PathBuf) -> Result<Connection> {
    std::fs::create_dir_all(app_data_dir).ok();
    let db_path = app_data_dir.join("app.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    conn.execute_batch(SCHEMA)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN team_home TEXT");
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN team_away TEXT");
    let _ = conn.execute_batch("ALTER TABLE scout_files ADD COLUMN has_video INTEGER NOT NULL DEFAULT 0");
    let _ = conn.execute_batch("ALTER TABLE scout_play_rows ADD COLUMN cache_version INTEGER NOT NULL DEFAULT 1");
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN match_date TEXT");
    seed(&conn)?;
    Ok(conn)
}

pub fn get_associations(db: &Connection) -> Result<Vec<Association>> {
    let mut stmt = db.prepare("SELECT id, name FROM associations ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Association {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    rows.collect()
}

pub fn create_association(db: &Connection, name: &str) -> Result<Association> {
    db.execute("INSERT INTO associations (name) VALUES (?1)", params![name])?;
    let id = db.last_insert_rowid();
    Ok(Association {
        id,
        name: name.to_string(),
    })
}

pub fn get_or_create_association(db: &Connection, name: &str) -> Result<Association> {
    db.execute("INSERT OR IGNORE INTO associations (name) VALUES (?1)", params![name])?;
    db.query_row(
        "SELECT id, name FROM associations WHERE name = ?1",
        params![name],
        |row| Ok(Association { id: row.get(0)?, name: row.get(1)? }),
    )
}

pub fn rename_association(db: &Connection, id: i64, name: &str) -> Result<()> {
    db.execute(
        "UPDATE associations SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![name, id],
    )?;
    Ok(())
}

pub fn delete_association(db: &Connection, id: i64) -> Result<()> {
    db.execute("DELETE FROM associations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_seasons(db: &Connection, association_id: i64) -> Result<Vec<Season>> {
    let mut stmt = db.prepare(
        "SELECT id, association_id, name FROM seasons WHERE association_id = ?1 ORDER BY name",
    )?;
    let rows = stmt.query_map(params![association_id], |row| {
        Ok(Season {
            id: row.get(0)?,
            association_id: row.get(1)?,
            name: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn create_season(db: &Connection, association_id: i64, name: &str) -> Result<Season> {
    db.execute(
        "INSERT INTO seasons (association_id, name) VALUES (?1, ?2)",
        params![association_id, name],
    )?;
    let id = db.last_insert_rowid();
    Ok(Season {
        id,
        association_id,
        name: name.to_string(),
    })
}

pub fn get_or_create_season(db: &Connection, association_id: i64, name: &str) -> Result<Season> {
    db.execute(
        "INSERT OR IGNORE INTO seasons (association_id, name) VALUES (?1, ?2)",
        params![association_id, name],
    )?;
    db.query_row(
        "SELECT id, association_id, name FROM seasons WHERE association_id = ?1 AND name = ?2",
        params![association_id, name],
        |row| Ok(Season { id: row.get(0)?, association_id: row.get(1)?, name: row.get(2)? }),
    )
}

pub fn rename_season(db: &Connection, id: i64, name: &str) -> Result<()> {
    db.execute(
        "UPDATE seasons SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![name, id],
    )?;
    Ok(())
}

pub fn delete_season(db: &Connection, id: i64) -> Result<()> {
    db.execute("DELETE FROM seasons WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_matches(db: &Connection, season_id: i64) -> Result<Vec<Match>> {
    let mut stmt = db.prepare(
        "SELECT m.id, m.season_id, m.name, m.team_home, m.team_away,
                EXISTS(SELECT 1 FROM scout_files sf WHERE sf.match_id = m.id AND sf.has_video = 1) AS has_video,
                m.match_date
         FROM matches m
         WHERE m.season_id = ?1
          ORDER BY m.created_at DESC, m.name",
    )?;
    let rows = stmt.query_map(params![season_id], |row| {
        Ok(Match {
            id: row.get(0)?,
            season_id: row.get(1)?,
            name: row.get(2)?,
            team_home: row.get(3)?,
            team_away: row.get(4)?,
            has_video: row.get::<_, i64>(5)? == 1,
            match_date: row.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn create_match(db: &Connection, season_id: i64, name: &str) -> Result<Match> {
    db.execute(
        "INSERT INTO matches (season_id, name) VALUES (?1, ?2)",
        params![season_id, name],
    )?;
    let id = db.last_insert_rowid();
    Ok(Match {
        id,
        season_id,
        name: name.to_string(),
        team_home: None,
        team_away: None,
        has_video: false,
        match_date: None,
    })
}

fn unique_match_name(db: &Connection, season_id: i64, name: &str, preferred_suffix: Option<&str>) -> Result<String> {
    let count: i64 = db.query_row(
        "SELECT COUNT(*) FROM matches WHERE season_id = ?1 AND name = ?2",
        params![season_id, name],
        |row| row.get(0),
    )?;
    if count == 0 {
        return Ok(name.to_string());
    }

    if let Some(suffix) = preferred_suffix {
        let candidate = format!("{name} {suffix}");
        let dup: i64 = db.query_row(
            "SELECT COUNT(*) FROM matches WHERE season_id = ?1 AND name = ?2",
            params![season_id, candidate],
            |row| row.get(0),
        )?;
        if dup == 0 {
            return Ok(candidate);
        }
    }

    let mut num = 2;
    loop {
        let candidate = format!("{name} {num}");
        let count: i64 = db.query_row(
            "SELECT COUNT(*) FROM matches WHERE season_id = ?1 AND name = ?2",
            params![season_id, candidate],
            |row| row.get(0),
        )?;
        if count == 0 {
            return Ok(candidate);
        }
        num += 1;
    }
}

pub fn import_scout_file(
    db: &Connection,
    app_data_dir: &Path,
    source_path: &str,
    association_name: &str,
    season_name: &str,
    match_name: &str,
) -> Result<ImportedScoutFile> {
    let source = PathBuf::from(source_path);
    let original_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("source.dvw")
        .to_string();
    let source_format = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let file_size = std::fs::metadata(&source)
        .map(|metadata| metadata.len() as i64)
        .map_err(|_| rusqlite::Error::InvalidPath(source.clone()))?;

    let association = get_or_create_association(db, association_name)?;
    let season = get_or_create_season(db, association.id, season_name)?;

    let file_content = std::fs::read_to_string(&source)
        .map_err(|_| rusqlite::Error::InvalidPath(source.clone()))?;
    let metadata = parsers::dvw::parse_metadata(&file_content);
    let team_home = metadata.team_home;
    let team_away = metadata.team_away;
    let has_video = metadata.has_video;
    let match_date = metadata.match_date;
    let unique_name = unique_match_name(db, season.id, match_name, match_date.as_deref())?;

    db.execute(
        "INSERT INTO matches (season_id, name, team_home, team_away, match_date) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![season.id, unique_name, team_home, team_away, match_date],
    )?;
    let match_id = db.last_insert_rowid();
    let match_record = Match {
        id: match_id,
        season_id: season.id,
        name: unique_name,
        team_home,
        team_away,
        has_video,
        match_date,
    };

    db.execute(
        "INSERT INTO scout_files (match_id, original_name, stored_path, source_path, file_size, source_format, has_video)
         VALUES (?1, ?2, '', ?3, ?4, ?5, ?6)",
        params![match_record.id, original_name, source_path, file_size, source_format, if has_video { 1 } else { 0 }],
    )?;
    let scout_file_id = db.last_insert_rowid();

    let target_dir = app_data_dir.join("scout-files").join(scout_file_id.to_string());
    std::fs::create_dir_all(&target_dir)
        .map_err(|_| rusqlite::Error::InvalidPath(target_dir.clone()))?;
    let target = target_dir.join(format!("original.{source_format}"));
    std::fs::copy(&source, &target)
        .map_err(|_| rusqlite::Error::InvalidPath(source.clone()))?;

    let stored_path = target.to_string_lossy().to_string();
    db.execute(
        "UPDATE scout_files SET stored_path = ?1 WHERE id = ?2",
        params![stored_path, scout_file_id],
    )?;
    rebuild_scout_row_cache(db, match_record.id, &stored_path)?;

    let scout_file = ScoutFile {
        id: scout_file_id,
        match_id: match_record.id,
        original_name,
        stored_path,
        source_path: Some(source_path.to_string()),
        file_size,
        source_format,
    };

    Ok(ImportedScoutFile { association, season, match_record, scout_file })
}

pub fn import_scout_files_batch(
    db: &Connection,
    app_data_dir: &Path,
    source_paths: &[String],
    association_name: &str,
    fallback_season_name: &str,
    auto_season: bool,
) -> Result<BatchImportResult> {
    let association = get_or_create_association(db, association_name)?;
    let mut imported = Vec::new();
    let mut failed = Vec::new();

    for source_path in source_paths {
        if !source_path.to_ascii_lowercase().ends_with(".dvw") {
            failed.push(BatchImportFailure {
                source_path: source_path.clone(),
                reason: "Only .dvw scout files are supported right now".to_string(),
            });
            continue;
        }

        let source = PathBuf::from(source_path);
        let original_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("source.dvw")
            .to_string();
        let filename_stem = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        let source_format = source
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let file_size = match std::fs::metadata(&source) {
            Ok(m) => m.len() as i64,
            Err(e) => {
                failed.push(BatchImportFailure {
                    source_path: source_path.clone(),
                    reason: format!("Unable to read file metadata: {e}"),
                });
                continue;
            }
        };

        let file_content = match std::fs::read_to_string(&source) {
            Ok(c) => c,
            Err(e) => {
                failed.push(BatchImportFailure {
                    source_path: source_path.clone(),
                    reason: format!("Unable to read file contents: {e}"),
                });
                continue;
            }
        };
        let metadata = parsers::dvw::parse_metadata(&file_content);

        let season_name = if auto_season {
            metadata.season_year.as_deref().unwrap_or(fallback_season_name)
        } else {
            fallback_season_name
        };

        let match_name = parsers::dvw::derive_match_name(
            metadata.team_home.as_deref(),
            metadata.team_away.as_deref(),
            &filename_stem,
        );

        let import_result: std::result::Result<ImportedScoutFile, String> = (|| {
            let season = get_or_create_season(db, association.id, season_name).map_err(|e| e.to_string())?;
            let team_home = metadata.team_home.clone();
            let team_away = metadata.team_away.clone();
            let has_video = metadata.has_video;
            let match_date = metadata.match_date.clone();
            let unique_name = unique_match_name(db, season.id, &match_name, match_date.as_deref()).map_err(|e| e.to_string())?;
            db.execute(
                "INSERT INTO matches (season_id, name, team_home, team_away, match_date) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![season.id, unique_name, team_home, team_away, match_date],
            )
            .map_err(|e| e.to_string())?;

            let match_id = db.last_insert_rowid();
            let match_record = Match {
                id: match_id,
                season_id: season.id,
                name: unique_name,
                team_home: metadata.team_home,
                team_away: metadata.team_away,
                has_video,
                match_date,
            };

            db.execute(
                "INSERT INTO scout_files (match_id, original_name, stored_path, source_path, file_size, source_format, has_video)
                 VALUES (?1, ?2, '', ?3, ?4, ?5, ?6)",
                params![match_record.id, original_name, source_path, file_size, source_format, if has_video { 1 } else { 0 }],
            )
            .map_err(|e| e.to_string())?;

            let scout_file_id = db.last_insert_rowid();
            let target_dir = app_data_dir.join("scout-files").join(scout_file_id.to_string());
            std::fs::create_dir_all(&target_dir).map_err(|e| {
                let _ = db.execute("DELETE FROM matches WHERE id = ?1", params![match_id]);
                format!("Unable to create target directory: {e}")
            })?;

            let target = target_dir.join(format!("original.{source_format}"));
            std::fs::copy(&source, &target).map_err(|e| {
                let _ = db.execute("DELETE FROM matches WHERE id = ?1", params![match_id]);
                format!("Unable to copy file into app storage: {e}")
            })?;

            let stored_path = target.to_string_lossy().to_string();
            db.execute(
                "UPDATE scout_files SET stored_path = ?1 WHERE id = ?2",
                params![stored_path, scout_file_id],
            )
            .map_err(|e| {
                let _ = db.execute("DELETE FROM matches WHERE id = ?1", params![match_id]);
                e.to_string()
            })?;
            rebuild_scout_row_cache(db, match_record.id, &stored_path).map_err(|e| {
                let _ = db.execute("DELETE FROM matches WHERE id = ?1", params![match_id]);
                e.to_string()
            })?;

            let scout_file = ScoutFile {
                id: scout_file_id,
                match_id: match_record.id,
                original_name,
                stored_path,
                source_path: Some(source_path.to_string()),
                file_size,
                source_format,
            };

            Ok(ImportedScoutFile {
                association: association.clone(),
                season,
                match_record,
                scout_file,
            })
        })();

        match import_result {
            Ok(result) => imported.push(result),
            Err(reason) => failed.push(BatchImportFailure {
                source_path: source_path.clone(),
                reason,
            }),
        }
    }

    Ok(BatchImportResult { imported, failed })
}

pub fn get_scout_lines(db: &Connection, match_id: i64) -> Result<Vec<String>> {
    let stored_path: String = db.query_row(
        "SELECT stored_path FROM scout_files WHERE match_id = ?1 ORDER BY id DESC LIMIT 1",
        params![match_id],
        |row| row.get(0),
    )?;

    let content = std::fs::read_to_string(&stored_path)
        .map_err(|_| rusqlite::Error::InvalidPath(PathBuf::from(stored_path)))?;
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let marker_index = lines.iter().position(|line| line.trim() == "[3SCOUT]");

    match marker_index {
        Some(index) => Ok(lines.into_iter().skip(index + 1).collect()),
        None => Ok(Vec::new()),
    }
}

pub fn get_scout_rows(db: &Connection, match_id: i64) -> Result<Vec<ScoutPlayRow>> {
    if scout_row_cache_count(db, match_id)? > 0
        && scout_row_cache_version(db, match_id)? >= SCOUT_ROW_CACHE_VERSION
        && scout_row_cache_skill_count(db, match_id)? > 0
        && scout_row_cache_bad_grade_count(db, match_id)? == 0
    {
        return get_cached_scout_rows(db, match_id);
    }

    let stored_path: String = db.query_row(
        "SELECT stored_path FROM scout_files WHERE match_id = ?1 ORDER BY id DESC LIMIT 1",
        params![match_id],
        |row| row.get(0),
    )?;

    rebuild_scout_row_cache(db, match_id, &stored_path)?;
    if scout_row_cache_count(db, match_id)? > 0 {
        return get_cached_scout_rows(db, match_id);
    }

    let content = std::fs::read_to_string(&stored_path)
        .map_err(|_| rusqlite::Error::InvalidPath(PathBuf::from(stored_path)))?;
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let marker_index = lines.iter().position(|line| line.trim() == "[3SCOUT]");

    match marker_index {
        Some(index) => {
            let mut rows = Vec::new();
            let mut current_home_score = "0".to_string();
            let mut current_away_score = "0".to_string();
            for (offset, line) in lines.iter().skip(index + 1).enumerate() {
                if line.trim().is_empty() {
                    continue;
                }
                if line.trim_start().starts_with('[') {
                    break;
                }
                let mut row = parse_scout_row((offset + 1) as i64, line);
                if let Some((home, away)) = parse_score_from_code(&row.code) {
                    current_home_score = home;
                    current_away_score = away;
                }
                row.score = Some(format!("{}-{}", current_home_score, current_away_score));
                rows.push(row);
            }
            Ok(rows)
        }
        None => Ok(Vec::new()),
    }
}

fn scout_row_cache_count(db: &Connection, match_id: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM scout_play_rows WHERE match_id = ?1",
        params![match_id],
        |row| row.get(0),
    )
}

fn get_cached_scout_rows(db: &Connection, match_id: i64) -> Result<Vec<ScoutPlayRow>> {
    let mut stmt = db.prepare(
        "SELECT row_id, raw_line, code, set_number, score, video_time_raw, video_time_seconds
         FROM scout_play_rows
         WHERE match_id = ?1
         ORDER BY row_id",
    )?;
    let rows = stmt.query_map(params![match_id], |row| {
        Ok(ScoutPlayRow {
            row_id: row.get(0)?,
            raw_line: row.get(1)?,
            code: row.get(2)?,
            set_number: row.get(3)?,
            score: row.get(4)?,
            video_time_raw: row.get(5)?,
            video_time_seconds: row.get(6)?,
            match_id: None,
            match_name: None,
        })
    })?;
    rows.collect()
}

fn rebuild_scout_row_cache(db: &Connection, match_id: i64, stored_path: &str) -> Result<()> {
    db.execute("DELETE FROM scout_play_rows WHERE match_id = ?1", params![match_id])?;
    let content = std::fs::read_to_string(stored_path)
        .map_err(|_| rusqlite::Error::InvalidPath(PathBuf::from(stored_path)))?;
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let Some(marker_index) = lines.iter().position(|line| line.trim() == "[3SCOUT]") else {
        return Ok(());
    };

    let mut current_home_score = "0".to_string();
    let mut current_away_score = "0".to_string();
    let mut rally_id = 0_i64;
    let mut insert = db.prepare(
        "INSERT INTO scout_play_rows (
            match_id, row_id, raw_line, code, set_number, score, video_time_raw, video_time_seconds,
            team, number, skill, sub_type, grade, combo, start_zone, end_zone, skill_type, players, rally_id, cache_version
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
    )?;

    for (offset, line) in lines.iter().skip(marker_index + 1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line.trim_start().starts_with('[') {
            break;
        }
        let mut scout_row = parse_scout_row((offset + 1) as i64, line);
        if let Some((home, away)) = parse_score_from_code(&scout_row.code) {
            current_home_score = home;
            current_away_score = away;
        }
        scout_row.score = Some(format!("{}-{}", current_home_score, current_away_score));

        let parsed = parse_code_fields(&scout_row.code);
        if parsed.skill == "S" {
            rally_id += 1;
        }
        insert.execute(params![
            match_id,
            scout_row.row_id,
            scout_row.raw_line,
            scout_row.code,
            scout_row.set_number,
            scout_row.score,
            scout_row.video_time_raw,
            scout_row.video_time_seconds,
            empty_as_none(&parsed.team),
            empty_as_none(&parsed.number),
            empty_as_none(&parsed.skill),
            empty_as_none(&parsed.sub_type),
            empty_as_none(&parsed.grade),
            empty_as_none(&parsed.combo),
            empty_as_none(&parsed.start_zone),
            empty_as_none(&parsed.end_zone),
            empty_as_none(&parsed.skill_type),
            empty_as_none(&parsed.players),
            rally_id,
            SCOUT_ROW_CACHE_VERSION,
        ])?;
    }

    Ok(())
}

pub fn get_scout_video_path(db: &Connection, match_id: i64) -> Result<Option<String>> {
    let (stored_path, source_path): (String, Option<String>) = db.query_row(
        "SELECT stored_path, source_path FROM scout_files WHERE match_id = ?1 ORDER BY id DESC LIMIT 1",
        params![match_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let content = std::fs::read_to_string(&stored_path)
        .map_err(|_| rusqlite::Error::InvalidPath(PathBuf::from(&stored_path)))?;
    let reference = match parsers::dvw::extract_video_reference(&content) {
        Some(v) => v,
        None => return Ok(None),
    };

    let normalized_reference = reference
        .trim()
        .trim_matches('"')
        .trim_start_matches("file://")
        .to_string();

    if normalized_reference.starts_with("http://") || normalized_reference.starts_with("https://") {
        return Ok(Some(normalized_reference));
    }

    let reference_path = PathBuf::from(&normalized_reference);
    if reference_path.is_absolute() && reference_path.exists() {
        return Ok(Some(normalized_reference));
    }

    if let Some(src) = source_path {
        let src_parent = PathBuf::from(src).parent().map(|p| p.to_path_buf());
        if let Some(parent) = src_parent {
            let candidate = parent.join(&normalized_reference);
            if candidate.exists() {
                return Ok(Some(candidate.to_string_lossy().to_string()));
            }
        }
    }

    let stored_parent = PathBuf::from(&stored_path).parent().map(|p| p.to_path_buf());
    if let Some(parent) = stored_parent {
        let candidate = parent.join(&normalized_reference);
        if candidate.exists() {
            return Ok(Some(candidate.to_string_lossy().to_string()));
        }
    }

    if reference_path.is_absolute() {
        return Ok(Some(normalized_reference));
    }

    Ok(None)
}

fn parse_scout_row(row_id: i64, line: &str) -> ScoutPlayRow {
    let parts: Vec<&str> = line.split(';').collect();
    let code = parts.first().map(|v| v.trim().to_string()).unwrap_or_default();

    let video_time_raw = parts
        .get(12)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let video_time_seconds = video_time_raw
        .as_deref()
        .and_then(parse_video_time_token);

    let set_number = parts
        .get(11)
        .map(|v| v.trim())
        .and_then(|v| v.parse::<i64>().ok())
        .or_else(|| parts.get(8).map(|v| v.trim()).and_then(|v| v.parse::<i64>().ok()));

    ScoutPlayRow {
        row_id,
        raw_line: line.to_string(),
        code,
        set_number,
        score: None,
        video_time_raw,
        video_time_seconds,
        match_id: None,
        match_name: None,
    }
}

fn parse_code_fields(code: &str) -> ParsedCodeFields {
    let normalized: String = code
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_uppercase())
        .collect();
    let main = normalized.split(';').next().unwrap_or("");
    let chars: Vec<char> = main.chars().collect();
    let mut fields = ParsedCodeFields::default();

    if chars.len() >= 4 {
        fields.team = chars[0].to_string();
        if chars[1].is_ascii_digit() && chars[2].is_ascii_digit() {
            fields.number = format!("{}{}", chars[1], chars[2]);
            fields.skill = chars[3].to_string();
            if chars.get(4).is_some_and(|c| is_grade_char(*c)) {
                fields.grade = chars[4].to_string();
                if chars.len() >= 7 && chars[5].is_ascii_alphabetic() && chars[6].is_ascii_digit() {
                    fields.combo = format!("{}{}", chars[5], chars[6]);
                }
            } else if chars.get(4).is_some_and(|c| c.is_ascii_alphabetic()) {
                fields.sub_type = chars[4].to_string();
                if chars.get(5).is_some_and(|c| is_grade_char(*c)) {
                    fields.grade = chars[5].to_string();
                }
            }
        }
    }

    if chars.get(12).is_some_and(|c| *c != '~') {
        fields.skill_type = chars[12].to_string();
    }
    if chars.get(13).is_some_and(|c| *c != '~') {
        fields.players = chars[13].to_string();
    }

    if let Some(tail) = main.rsplit('~').next() {
        let tail_chars: Vec<char> = tail.chars().collect();
        if tail_chars.len() >= 2 && tail_chars[0].is_ascii_digit() && tail_chars[1].is_ascii_digit() {
            fields.start_zone = tail_chars[0].to_string();
            fields.end_zone = tail_chars[1].to_string();
            if fields.skill_type.is_empty() && tail_chars.get(2).is_some_and(|c| c.is_ascii_alphabetic()) {
                fields.skill_type = tail_chars[2].to_string();
            }
            if tail_chars.get(3).is_some_and(|c| c.is_ascii_alphabetic()) {
                fields.sub_type = tail_chars[3].to_string();
            }
            if fields.players.is_empty() && tail_chars.get(4).is_some_and(|c| c.is_ascii_digit()) {
                fields.players = tail_chars[4].to_string();
            }
        }
    }

    if fields.skill == "R" {
        let reception_letters: Vec<String> = [fields.skill_type.clone(), fields.sub_type.clone()]
            .into_iter()
            .filter(|value| matches!(value.as_str(), "M" | "R" | "L" | "W"))
            .collect();
        if let Some(value) = reception_letters.get(0) {
            fields.skill_type = value.clone();
        }
        if let Some(value) = reception_letters.get(1) {
            fields.sub_type = value.clone();
        }
    }

    if fields.skill == "A" && fields.players == "3" {
        fields.players = "0".to_string();
    }

    fields
}

fn is_grade_char(value: char) -> bool {
    matches!(value, '#' | '!' | '+' | '-' | '/' | '=')
}

fn empty_as_none(value: &str) -> Option<&str> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn parse_video_time_token(token: &str) -> Option<f64> {
    let trimmed = token.trim();
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        trimmed.parse::<f64>().ok()
    } else {
        None
    }
}

fn parse_score_from_code(code: &str) -> Option<(String, String)> {
    if code.len() < 5 {
        return None;
    }
    let mut chars = code.chars();
    let first = chars.next()?;
    if first != '*' && first != 'a' {
        return None;
    }
    if chars.next()? != 'p' {
        return None;
    }
    let rest = chars.as_str();
    let sep = rest.find(':')?;
    let home = &rest[..sep];
    let away = &rest[sep + 1..];
    if home.is_empty() || away.is_empty() {
        return None;
    }
    if home.chars().all(|c| c.is_ascii_digit()) && away.chars().all(|c| c.is_ascii_digit()) {
        return Some((home.to_string(), away.to_string()));
    }
    None
}

pub fn rename_match(db: &Connection, id: i64, name: &str) -> Result<()> {
    db.execute(
        "UPDATE matches SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![name, id],
    )?;
    Ok(())
}

pub fn get_teams_for_seasons(db: &Connection, season_ids: &[i64]) -> Result<Vec<String>> {
    if season_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders: Vec<String> = season_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT DISTINCT name FROM (
            SELECT team_home AS name FROM matches WHERE season_id IN ({ph}) AND team_home IS NOT NULL
            UNION
            SELECT team_away AS name FROM matches WHERE season_id IN ({ph}) AND team_away IS NOT NULL
        ) ORDER BY name",
        ph = placeholders.join(",")
    );
    let params: Vec<&dyn rusqlite::ToSql> = season_ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    let mut stmt = db.prepare(&sql)?;
    let rows = stmt.query_map(params.as_slice(), |row| row.get(0))?;
    rows.collect()
}

pub fn delete_match(db: &Connection, id: i64) -> Result<()> {
    db.execute("DELETE FROM matches WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_match_name(db: &Connection, match_id: i64) -> Result<String> {
    db.query_row(
        "SELECT name FROM matches WHERE id = ?1",
        params![match_id],
        |row| row.get(0),
    )
}

pub fn get_scout_rows_multi(db: &Connection, match_ids: &[i64]) -> Result<Vec<ScoutPlayRow>> {
    let mut all_rows = Vec::new();
    for &mid in match_ids {
        let match_name = get_match_name(db, mid).unwrap_or_else(|_| format!("Match {mid}"));
        let mut rows = get_scout_rows(db, mid)?;
        for row in &mut rows {
            row.match_id = Some(mid);
            row.match_name = Some(match_name.clone());
        }
        all_rows.extend(rows);
    }
    Ok(all_rows)
}

pub fn get_scout_rows_multi_filtered(
    db: &Connection,
    match_ids: &[i64],
    filters: &[ScoutFilterRow],
) -> Result<Vec<ScoutPlayRow>> {
    if filters.iter().skip(1).all(|filter| !filter_has_fields(filter)) {
        if let Some(base_filter) = filters.first() {
            return get_scout_rows_multi_base_filtered(db, match_ids, base_filter);
        }
    }

    let mut all_rows = Vec::new();
    for &mid in match_ids {
        let match_name = get_match_name(db, mid).unwrap_or_else(|_| format!("Match {mid}"));
        let mut rows = get_scout_rows(db, mid)?;
        let filtered = filter_scout_rows(&rows, filters);
        for mut row in filtered {
            row.match_id = Some(mid);
            row.match_name = Some(match_name.clone());
            all_rows.push(row);
        }
        rows.clear();
    }
    Ok(all_rows)
}

fn get_scout_rows_multi_base_filtered(
    db: &Connection,
    match_ids: &[i64],
    filter: &ScoutFilterRow,
) -> Result<Vec<ScoutPlayRow>> {
    if match_ids.is_empty() {
        return Ok(Vec::new());
    }
    for &match_id in match_ids {
        ensure_scout_row_cache(db, match_id)?;
    }

    let mut sql = String::from(
        "SELECT r.row_id, r.raw_line, r.code, r.set_number, r.score, r.video_time_raw, r.video_time_seconds,
                r.match_id, m.name
         FROM scout_play_rows r
         JOIN matches m ON m.id = r.match_id
         WHERE r.match_id IN (",
    );
    sql.push_str(&vec!["?"; match_ids.len()].join(","));
    sql.push(')');

    let mut owned_params: Vec<String> = Vec::new();
    add_filter_clause(&mut sql, &mut owned_params, "r.team", &filter.team);
    add_filter_clause(&mut sql, &mut owned_params, "r.number", &filter.number);
    add_skill_filter_clause(&mut sql, &mut owned_params, &filter.skill);
    add_filter_clause(&mut sql, &mut owned_params, "r.sub_type", &filter.sub_type);
    add_filter_clause(&mut sql, &mut owned_params, "r.grade", &filter.grade);
    add_filter_clause(&mut sql, &mut owned_params, "r.combo", &filter.combo);
    add_filter_clause(&mut sql, &mut owned_params, "r.start_zone", &filter.start_zone);
    add_filter_clause(&mut sql, &mut owned_params, "r.end_zone", &filter.end_zone);
    add_filter_clause(&mut sql, &mut owned_params, "r.skill_type", &filter.skill_type);
    add_filter_clause(&mut sql, &mut owned_params, "r.players", &filter.players);
    sql.push_str(" ORDER BY r.match_id, r.row_id");

    let mut params_vec: Vec<&dyn ToSql> = match_ids.iter().map(|id| id as &dyn ToSql).collect();
    for value in &owned_params {
        params_vec.push(value as &dyn ToSql);
    }

    let mut stmt = db.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params_vec), |row| {
        Ok(ScoutPlayRow {
            row_id: row.get(0)?,
            raw_line: row.get(1)?,
            code: row.get(2)?,
            set_number: row.get(3)?,
            score: row.get(4)?,
            video_time_raw: row.get(5)?,
            video_time_seconds: row.get(6)?,
            match_id: row.get(7)?,
            match_name: row.get(8)?,
        })
    })?;
    rows.collect()
}

fn add_filter_clause(sql: &mut String, params: &mut Vec<String>, column: &str, query: &str) {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }
    sql.push_str(" AND ");
    sql.push_str(column);
    sql.push_str(" LIKE ?");
    params.push(format!("%{}%", trimmed.to_ascii_uppercase()));
}

fn add_skill_filter_clause(sql: &mut String, params: &mut Vec<String>, query: &str) {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return;
    }
    let skill = trimmed.to_ascii_uppercase();
    sql.push_str(" AND (r.skill LIKE ? OR r.code LIKE ?)");
    params.push(format!("%{}%", skill));
    params.push(format!("___{}%", skill));
}

fn ensure_scout_row_cache(db: &Connection, match_id: i64) -> Result<()> {
    if scout_row_cache_count(db, match_id)? > 0
        && scout_row_cache_version(db, match_id)? >= SCOUT_ROW_CACHE_VERSION
        && scout_row_cache_skill_count(db, match_id)? > 0
        && scout_row_cache_bad_grade_count(db, match_id)? == 0
    {
        return Ok(());
    }
    let stored_path: Option<String> = db.query_row(
        "SELECT stored_path FROM scout_files WHERE match_id = ?1 ORDER BY id DESC LIMIT 1",
        params![match_id],
        |row| row.get(0),
    ).optional()?;
    let Some(stored_path) = stored_path else {
        return Ok(());
    };
    rebuild_scout_row_cache(db, match_id, &stored_path)
}

fn scout_row_cache_skill_count(db: &Connection, match_id: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM scout_play_rows WHERE match_id = ?1 AND skill IS NOT NULL AND skill != ''",
        params![match_id],
        |row| row.get(0),
    )
}

fn scout_row_cache_version(db: &Connection, match_id: i64) -> Result<i64> {
    db.query_row(
        "SELECT COALESCE(MIN(cache_version), 0) FROM scout_play_rows WHERE match_id = ?1",
        params![match_id],
        |row| row.get(0),
    )
}

fn scout_row_cache_bad_grade_count(db: &Connection, match_id: i64) -> Result<i64> {
    db.query_row(
        "SELECT COUNT(*) FROM scout_play_rows
         WHERE match_id = ?1
           AND grade IS NOT NULL
           AND grade != ''
           AND grade NOT IN ('#', '!', '+', '-', '/', '=')",
        params![match_id],
        |row| row.get(0),
    )
}

fn filter_scout_rows(rows: &[ScoutPlayRow], filters: &[ScoutFilterRow]) -> Vec<ScoutPlayRow> {
    if filters.iter().all(|filter| !filter_has_fields(filter)) {
        return rows.to_vec();
    }
    let action_rows: Vec<ScoutPlayRow> = rows
        .iter()
        .filter(|row| is_action_row(row))
        .cloned()
        .collect();
    let Some(base_filter) = filters.first() else {
        return action_rows;
    };
    let chain_filters: Vec<&ScoutFilterRow> = filters
        .iter()
        .skip(1)
        .filter(|filter| filter_has_fields(filter))
        .collect();

    action_rows
        .iter()
        .enumerate()
        .filter_map(|(idx, row)| {
            if !row_matches_condition(row, base_filter) {
                return None;
            }
            for filter in &chain_filters {
                if filter.relation.as_deref() == Some("rally_contains") {
                    if !rally_contains(&action_rows, idx, filter) {
                        return None;
                    }
                    continue;
                }
                let target_idx = idx as isize + relation_offset(filter.relation.as_deref());
                if target_idx < 0 {
                    return None;
                }
                let target = action_rows.get(target_idx as usize)?;
                if !row_matches_condition(target, filter) {
                    return None;
                }
            }
            Some(row.clone())
        })
        .collect()
}

fn filter_has_fields(filter: &ScoutFilterRow) -> bool {
    [
        &filter.team,
        &filter.number,
        &filter.skill,
        &filter.sub_type,
        &filter.grade,
        &filter.combo,
        &filter.start_zone,
        &filter.end_zone,
        &filter.skill_type,
        &filter.players,
    ]
    .iter()
    .any(|value| !value.trim().is_empty())
}

fn row_matches_condition(row: &ScoutPlayRow, filter: &ScoutFilterRow) -> bool {
    let matched = matches_filter_row(row, filter);
    if filter.condition.as_deref() == Some("not_contains") || filter.relation.as_deref() == Some("not_equal") {
        !matched
    } else {
        matched
    }
}

fn matches_filter_row(row: &ScoutPlayRow, filter: &ScoutFilterRow) -> bool {
    let parsed = parse_code_fields(&row.code);
    matches_field(&parsed.team, &filter.team)
        && matches_field(&parsed.number, &filter.number)
        && matches_field(&parsed.skill, &filter.skill)
        && matches_field(&parsed.sub_type, &filter.sub_type)
        && matches_field(&parsed.grade, &filter.grade)
        && matches_field(&parsed.combo, &filter.combo)
        && matches_field(&parsed.start_zone, &filter.start_zone)
        && matches_field(&parsed.end_zone, &filter.end_zone)
        && matches_field(&parsed.skill_type, &filter.skill_type)
        && matches_field(&parsed.players, &filter.players)
}

fn matches_field(value: &str, query: &str) -> bool {
    let q = query.trim().to_ascii_uppercase();
    q.is_empty() || value.contains(&q)
}

fn is_action_row(row: &ScoutPlayRow) -> bool {
    let parsed = parse_code_fields(&row.code);
    !parsed.team.is_empty() && !parsed.number.is_empty() && !parsed.skill.is_empty()
}

fn relation_offset(relation: Option<&str>) -> isize {
    match relation {
        Some("previous") => -1,
        Some("previous_previous") => -2,
        Some("next_next") => 2,
        _ => 1,
    }
}

fn rally_contains(action_rows: &[ScoutPlayRow], row_idx: usize, filter: &ScoutFilterRow) -> bool {
    let mut start = row_idx;
    while start > 0 {
        if parse_code_fields(&action_rows[start - 1].code).skill == "S" {
            break;
        }
        start -= 1;
    }
    let mut end = row_idx;
    while end + 1 < action_rows.len() {
        if parse_code_fields(&action_rows[end + 1].code).skill == "S" {
            break;
        }
        end += 1;
    }
    (start..=end).any(|idx| idx != row_idx && row_matches_condition(&action_rows[idx], filter))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoPathEntry {
    pub match_id: i64,
    pub video_path: Option<String>,
}

pub fn get_scout_video_paths_multi(db: &Connection, match_ids: &[i64]) -> Result<Vec<VideoPathEntry>> {
    let mut entries = Vec::new();
    for &mid in match_ids {
        let path = get_scout_video_path(db, mid)?;
        entries.push(VideoPathEntry {
            match_id: mid,
            video_path: path,
        });
    }
    Ok(entries)
}

pub fn get_setting(db: &Connection, key: &str) -> Result<String> {
    db.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| {
        row.get(0)
    })
}

pub fn set_setting(db: &Connection, key: &str, value: &str) -> Result<()> {
    db.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )?;
    Ok(())
}
