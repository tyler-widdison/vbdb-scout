mod db;
mod parsers;

use db::AppState;
use rusqlite::Error as SqlError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, Manager};

#[tauri::command]
fn init_app(app: tauri::AppHandle) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let conn = db::init_db(&data_dir).map_err(|e| e.to_string())?;
    let state = AppState {
        db: std::sync::Mutex::new(conn),
        app_data_dir: data_dir,
    };
    app.manage(state);
    Ok(())
}

fn get_db<'a>(app: &'a tauri::AppHandle) -> Result<tauri::State<'a, AppState>, String> {
    app.try_state::<AppState>().ok_or("DB not initialized".into())
}

#[tauri::command]
fn get_associations(app: tauri::AppHandle) -> Result<Vec<db::Association>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_associations(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_association(app: tauri::AppHandle, name: String) -> Result<db::Association, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::create_association(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_association(app: tauri::AppHandle, id: i64, name: String) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::rename_association(&conn, id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_association(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_association(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_seasons(app: tauri::AppHandle, association_id: i64) -> Result<Vec<db::Season>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_seasons(&conn, association_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_season(
    app: tauri::AppHandle,
    association_id: i64,
    name: String,
) -> Result<db::Season, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::create_season(&conn, association_id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_season(app: tauri::AppHandle, id: i64, name: String) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::rename_season(&conn, id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_season(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_season(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_matches(app: tauri::AppHandle, season_id: i64) -> Result<Vec<db::Match>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_matches(&conn, season_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_match(
    app: tauri::AppHandle,
    season_id: i64,
    name: String,
) -> Result<db::Match, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::create_match(&conn, season_id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_match(app: tauri::AppHandle, id: i64, name: String) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::rename_match(&conn, id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_match(app: tauri::AppHandle, id: i64) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_match(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_scout_file(
    app: tauri::AppHandle,
    source_path: String,
    association_name: String,
    season_name: String,
    match_name: String,
) -> Result<db::ImportedScoutFile, String> {
    if !source_path.to_ascii_lowercase().ends_with(".dvw") {
        return Err("Only .dvw scout files are supported right now".into());
    }

    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::import_scout_file(
        &conn,
        &state.app_data_dir,
        &source_path,
        association_name.trim(),
        season_name.trim(),
        match_name.trim(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scout_lines(app: tauri::AppHandle, match_id: i64) -> Result<Vec<String>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_lines(&conn, match_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scout_rows(app: tauri::AppHandle, match_id: i64) -> Result<Vec<db::ScoutPlayRow>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_rows(&conn, match_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_scout_codes(
    app: tauri::AppHandle,
    changes: Vec<db::ScoutCodeChange>,
) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_scout_codes(&conn, &changes).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_stored_scout_files_path(app: tauri::AppHandle) -> Result<String, String> {
    let state = get_db(&app)?;
    Ok(state
        .app_data_dir
        .join("scout-files")
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
fn get_scout_video_path(app: tauri::AppHandle, match_id: i64) -> Result<Option<String>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_video_path(&conn, match_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scout_rows_multi(app: tauri::AppHandle, match_ids: Vec<i64>) -> Result<Vec<db::ScoutPlayRow>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_rows_multi(&conn, &match_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scout_rows_multi_filtered(
    app: tauri::AppHandle,
    match_ids: Vec<i64>,
    filters: Vec<db::ScoutFilterRow>,
) -> Result<Vec<db::ScoutPlayRow>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_rows_multi_filtered(&conn, &match_ids, &filters).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_scout_video_paths_multi(app: tauri::AppHandle, match_ids: Vec<i64>) -> Result<Vec<db::VideoPathEntry>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scout_video_paths_multi(&conn, &match_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_teams_for_seasons(app: tauri::AppHandle, season_ids: Vec<i64>) -> Result<Vec<String>, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_teams_for_seasons(&conn, &season_ids).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_scout_files(
    app: tauri::AppHandle,
    source_paths: Vec<String>,
    association_name: String,
    fallback_season_name: String,
    auto_season: bool,
) -> Result<db::BatchImportResult, String> {
    if source_paths.is_empty() {
        return Err("No source files were provided".into());
    }
    if source_paths
        .iter()
        .any(|path| !path.to_ascii_lowercase().ends_with(".dvw"))
    {
        return Err("Only .dvw scout files are supported right now".into());
    }

    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::import_scout_files_batch(
        &conn,
        &state.app_data_dir,
        &source_paths,
        association_name.trim(),
        fallback_season_name.trim(),
        auto_season,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_auto_season(app: tauri::AppHandle) -> Result<bool, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match db::get_setting(&conn, "auto_season") {
        Ok(val) => Ok(val == "true"),
        Err(SqlError::QueryReturnedNoRows) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn set_auto_season(app: tauri::AppHandle, value: bool) -> Result<(), String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::set_setting(&conn, "auto_season", if value { "true" } else { "false" })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_file_type(app: tauri::AppHandle) -> Result<String, String> {
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_setting(&conn, "file_type").map_err(|e| e.to_string())
}

#[tauri::command]
fn set_file_type(app: tauri::AppHandle, file_type: String) -> Result<(), String> {
    let allowed = ["json", "txt", "csv", "dvw"];
    if !allowed.contains(&file_type.as_str()) {
        return Err(format!("Invalid file type: {file_type}"));
    }
    let state = get_db(&app)?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::set_setting(&conn, "file_type", &file_type).map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
struct MontageClipInput {
    row_id: i64,
    match_id: Option<i64>,
    match_name: Option<String>,
    video_path: String,
    start_time: f64,
    end_time: f64,
    code: String,
    video_time_seconds: f64,
}

#[derive(Debug, Serialize)]
struct ExportMontageVideoResult {
    output_path: String,
    clips_exported: usize,
}

#[derive(Debug, Clone, Serialize)]
struct MontageExportProgress {
    phase: String,
    current: usize,
    total: usize,
}

#[tauri::command]
fn export_montage_video(
    app: tauri::AppHandle,
    output_path: String,
    clips: Vec<MontageClipInput>,
) -> Result<ExportMontageVideoResult, String> {
    if clips.is_empty() {
        return Err("No clips selected for export".into());
    }

    let output = PathBuf::from(output_path.clone());
    let output_parent = output
        .parent()
        .ok_or_else(|| "Output path has no parent directory".to_string())?;
    if !output_parent.exists() {
        return Err("Output directory does not exist".into());
    }

    let temp_root = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("montage-export-temp");
    fs::create_dir_all(&temp_root).map_err(|e| e.to_string())?;
    let work_dir = temp_root.join(format!(
        "job-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis()
    ));
    fs::create_dir_all(&work_dir).map_err(|e| e.to_string())?;

    let mut segment_paths: Vec<PathBuf> = Vec::new();
    let total_steps = clips.len() + 1;
    for (index, clip) in clips.iter().enumerate() {
        let _ = app.emit("montage-export-progress", MontageExportProgress {
            phase: "clip".to_string(),
            current: index + 1,
            total: total_steps,
        });
        let _ = (&clip.row_id, &clip.match_id, &clip.match_name, &clip.code, &clip.video_time_seconds);
        let input = Path::new(&clip.video_path);
        if !input.exists() {
            return Err(format!("Clip source file does not exist: {}", clip.video_path));
        }
        let start = clip.start_time.max(0.0);
        let end = clip.end_time.max(0.0);
        if end <= start {
            return Err(format!("Invalid clip range at row {}", clip.row_id));
        }
        let duration = end - start;
        let segment_path = work_dir.join(format!("segment-{index:05}.mp4"));
        run_ffmpeg(&app, [
            "-y".to_string(),
            "-ss".to_string(),
            format!("{start:.3}"),
            "-i".to_string(),
            clip.video_path.clone(),
            "-t".to_string(),
            format!("{duration:.3}"),
            "-vf".to_string(),
            "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2:color=black,fps=30".to_string(),
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "veryfast".to_string(),
            "-crf".to_string(),
            "23".to_string(),
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
            "-ar".to_string(),
            "48000".to_string(),
            "-ac".to_string(),
            "2".to_string(),
            segment_path.to_string_lossy().to_string(),
        ])?;
        segment_paths.push(segment_path);
    }

    let concat_list_path = work_dir.join("concat.txt");
    let concat_body = segment_paths
        .iter()
        .map(|p| format!("file '{}'", p.to_string_lossy()))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&concat_list_path, concat_body).map_err(|e| e.to_string())?;

    let _ = app.emit("montage-export-progress", MontageExportProgress {
        phase: "concat".to_string(),
        current: total_steps,
        total: total_steps,
    });
    run_ffmpeg(&app, [
        "-y".to_string(),
        "-f".to_string(),
        "concat".to_string(),
        "-safe".to_string(),
        "0".to_string(),
        "-i".to_string(),
        concat_list_path.to_string_lossy().to_string(),
        "-c:v".to_string(),
        "libx264".to_string(),
        "-preset".to_string(),
        "veryfast".to_string(),
        "-crf".to_string(),
        "23".to_string(),
        "-pix_fmt".to_string(),
        "yuv420p".to_string(),
        "-c:a".to_string(),
        "aac".to_string(),
        "-b:a".to_string(),
        "128k".to_string(),
        output_path.clone(),
    ])?;

    let _ = fs::remove_dir_all(&work_dir);
    let _ = app.emit("montage-export-progress", MontageExportProgress {
        phase: "done".to_string(),
        current: total_steps,
        total: total_steps,
    });
    Ok(ExportMontageVideoResult {
        output_path,
        clips_exported: clips.len(),
    })
}

fn run_ffmpeg<I>(app: &tauri::AppHandle, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = String>,
{
    let args_vec: Vec<String> = args.into_iter().collect();
    let candidates = ffmpeg_candidates(app);
    let mut launch_errors: Vec<String> = Vec::new();

    for candidate in &candidates {
        match Command::new(candidate).args(&args_vec).output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("ffmpeg failed: {stderr}"));
            }
            Err(e) => {
                launch_errors.push(format!("{} ({e})", candidate.display()));
            }
        }
    }

    Err(format!(
        "Could not launch ffmpeg. Tried: {}",
        launch_errors.join(", ")
    ))
}

fn ffmpeg_candidates(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut candidates = vec![PathBuf::from("ffmpeg")];

    if let Ok(resource_dir) = app.path().resource_dir() {
        #[cfg(target_os = "windows")]
        {
            candidates.push(resource_dir.join("ffmpeg").join("ffmpeg.exe"));
            candidates.push(resource_dir.join("bin").join("ffmpeg.exe"));
            candidates.push(resource_dir.join("ffmpeg.exe"));
        }
        #[cfg(target_os = "macos")]
        {
            candidates.push(resource_dir.join("ffmpeg").join("ffmpeg"));
            candidates.push(resource_dir.join("bin").join("ffmpeg"));
            candidates.push(resource_dir.join("ffmpeg"));
        }
        #[cfg(target_os = "linux")]
        {
            candidates.push(resource_dir.join("ffmpeg").join("ffmpeg"));
            candidates.push(resource_dir.join("bin").join("ffmpeg"));
            candidates.push(resource_dir.join("ffmpeg"));
        }
    }

    candidates
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            init_app,
            get_associations,
            create_association,
            rename_association,
            delete_association,
            get_seasons,
            create_season,
            rename_season,
            delete_season,
            get_matches,
            create_match,
            rename_match,
            delete_match,
            import_scout_file,
            import_scout_files,
            get_scout_lines,
            get_scout_rows,
            update_scout_codes,
            get_scout_rows_multi,
            get_scout_rows_multi_filtered,
            get_scout_video_path,
            get_scout_video_paths_multi,
            get_teams_for_seasons,
            get_auto_season,
            set_auto_season,
            get_file_type,
            set_file_type,
            get_stored_scout_files_path,
            export_montage_video,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
