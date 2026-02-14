use std::path::PathBuf;

use rusqlite::{params, params_from_iter, OptionalExtension};
use tauri::{Emitter, State};

use crate::clipboard;
use crate::db;
use crate::error::AppError;
use crate::models::{
  AppInfo, GalleryImage, HealthStatus, ImagesPage, ImportProgress, ImportSummary, TagItem,
  TagMutationResult,
};
use crate::scanner::{self, ScannedFile};
use crate::search;
use crate::tags;
use crate::thumbnail;

#[derive(Debug)]
pub struct AppState {
  pub app_data_dir: PathBuf,
  pub database_path: PathBuf,
}

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppInfo {
  AppInfo {
    app_data_dir: state.app_data_dir.to_string_lossy().to_string(),
    database_path: state.database_path.to_string_lossy().to_string(),
  }
}

#[tauri::command]
pub fn health_check(state: State<'_, AppState>) -> Result<HealthStatus, String> {
  run_health_check(&state).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_folder(
  app: tauri::AppHandle,
  state: State<'_, AppState>,
  path: String,
) -> Result<ImportSummary, String> {
  run_import_folder(&app, &state, &path).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_images(
  state: State<'_, AppState>,
  page: Option<i64>,
  limit: Option<i64>,
  search: Option<String>,
) -> Result<ImagesPage, String> {
  let resolved_page = page.unwrap_or(1).max(1);
  let resolved_limit = limit.unwrap_or(120).clamp(1, 500);
  run_get_images(&state, resolved_page, resolved_limit, search.as_deref())
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_recent_images(
  state: State<'_, AppState>,
  limit: Option<i64>,
) -> Result<Vec<GalleryImage>, String> {
  let resolved_limit = limit.unwrap_or(20).clamp(1, 80);
  run_get_recent_images(&state, resolved_limit).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_image_path(state: State<'_, AppState>, image_id: i64) -> Result<String, String> {
  run_get_image_path(&state, image_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn mark_image_used(state: State<'_, AppState>, image_id: i64) -> Result<(), String> {
  run_mark_image_used(&state, image_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn copy_image_to_clipboard(state: State<'_, AppState>, image_id: i64) -> Result<(), String> {
  run_copy_image_to_clipboard(&state, image_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_tags(
  state: State<'_, AppState>,
  query: Option<String>,
  limit: Option<i64>,
) -> Result<Vec<TagItem>, String> {
  let resolved_limit = limit.unwrap_or(80).clamp(1, 200);
  run_get_tags(&state, query.as_deref(), resolved_limit).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_image_tags(state: State<'_, AppState>, image_id: i64) -> Result<Vec<TagItem>, String> {
  run_get_image_tags(&state, image_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn create_tag(
  state: State<'_, AppState>,
  name: String,
  color: Option<String>,
  parent_id: Option<i64>,
) -> Result<TagItem, String> {
  run_create_tag(&state, &name, color, parent_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
  run_delete_tag(&state, tag_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn add_tags_to_images(
  state: State<'_, AppState>,
  image_ids: Vec<i64>,
  tag_names: Vec<String>,
) -> Result<TagMutationResult, String> {
  run_add_tags_to_images(&state, &image_ids, &tag_names).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn remove_tags_from_images(
  state: State<'_, AppState>,
  image_ids: Vec<i64>,
  tag_names: Vec<String>,
) -> Result<TagMutationResult, String> {
  run_remove_tags_from_images(&state, &image_ids, &tag_names).map_err(|error| error.to_string())
}

fn run_health_check(state: &AppState) -> Result<HealthStatus, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  let image_count = connection.query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))?;
  let tag_count = connection.query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;

  Ok(HealthStatus {
    ok: true,
    image_count,
    tag_count,
  })
}

fn run_import_folder(
  app: &tauri::AppHandle,
  state: &AppState,
  folder_path: &str,
) -> Result<ImportSummary, AppError> {
  let trimmed_path = folder_path.trim();
  if trimmed_path.is_empty() {
    return Err(AppError::InvalidInput(
      "import path cannot be empty".to_string(),
    ));
  }

  let root = PathBuf::from(trimmed_path);
  let scanned_files = scanner::collect_images(&root)?;

  let total = scanned_files.len();
  let mut imported = 0_usize;
  let mut skipped = 0_usize;
  let mut failed = 0_usize;

  let mut connection = db::open_connection(&state.database_path)?;
  let transaction = connection.transaction()?;

  {
    let mut existing_hash_stmt =
      transaction.prepare("SELECT 1 FROM images WHERE hash = ?1 LIMIT 1")?;
    let mut insert_stmt = transaction.prepare(
      "INSERT INTO images (
          filename,
          filepath,
          hash,
          width,
          height,
          file_size,
          mime_type,
          thumb_path
      ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;

    for (index, file) in scanned_files.iter().enumerate() {
      match import_single_image(
        file,
        &state.app_data_dir,
        &mut existing_hash_stmt,
        &mut insert_stmt,
      ) {
        Ok(ImportOutcome::Imported) => imported += 1,
        Ok(ImportOutcome::Skipped) => skipped += 1,
        Err(_error) => failed += 1,
      }

      let progress = ImportProgress {
        current: index + 1,
        total,
        imported,
        skipped,
        failed,
        current_path: Some(file.path.to_string_lossy().to_string()),
      };

      app.emit("import-progress", &progress)?;
    }
  }

  transaction.commit()?;

  app.emit(
    "import-progress",
    ImportProgress {
      current: total,
      total,
      imported,
      skipped,
      failed,
      current_path: None,
    },
  )?;

  Ok(ImportSummary {
    imported,
    skipped,
    failed,
    total,
  })
}

fn run_get_images(
  state: &AppState,
  page: i64,
  limit: i64,
  search_query: Option<&str>,
) -> Result<ImagesPage, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  let offset = (page - 1) * limit;

  let filter = search::build_filter(search_query);

  let total = if let Some(active_filter) = &filter {
    let total_sql = format!("SELECT COUNT(*) FROM images i WHERE {}", active_filter.clause);
    connection.query_row(&total_sql, params_from_iter(active_filter.params.iter()), |row| {
      row.get(0)
    })?
  } else {
    connection.query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))?
  };

  let mut items = Vec::new();

  if let Some(active_filter) = filter {
    let sql = format!(
      "SELECT
          i.id,
          i.filename,
          i.filepath,
          i.thumb_path,
          i.width,
          i.height,
          i.file_size,
          i.mime_type,
          i.imported_at,
          i.last_used_at
       FROM images i
       WHERE {}
       ORDER BY (i.last_used_at IS NULL), i.last_used_at DESC, i.imported_at DESC, i.id DESC
       LIMIT {} OFFSET {}",
      active_filter.clause, limit, offset
    );

    let mut statement = connection.prepare(&sql)?;
    let rows = statement.query_map(params_from_iter(active_filter.params.iter()), map_gallery_row)?;

    for row in rows {
      let mut image = row?;
      image.thumb_file_path = image
        .thumb_path
        .as_ref()
        .map(|relative| state.app_data_dir.join(relative).to_string_lossy().to_string());
      items.push(image);
    }
  } else {
    let mut statement = connection.prepare(
      "SELECT
          id,
          filename,
          filepath,
          thumb_path,
          width,
          height,
          file_size,
          mime_type,
          imported_at,
          last_used_at
       FROM images
       ORDER BY (last_used_at IS NULL), last_used_at DESC, imported_at DESC, id DESC
       LIMIT ?1 OFFSET ?2",
    )?;

    let rows = statement.query_map(params![limit, offset], map_gallery_row)?;

    for row in rows {
      let mut image = row?;
      image.thumb_file_path = image
        .thumb_path
        .as_ref()
        .map(|relative| state.app_data_dir.join(relative).to_string_lossy().to_string());
      items.push(image);
    }
  }

  Ok(ImagesPage {
    items,
    total,
    page,
    limit,
  })
}

fn run_get_recent_images(state: &AppState, limit: i64) -> Result<Vec<GalleryImage>, AppError> {
  let connection = db::open_connection(&state.database_path)?;

  let mut statement = connection.prepare(
    "SELECT
        id,
        filename,
        filepath,
        thumb_path,
        width,
        height,
        file_size,
        mime_type,
        imported_at,
        last_used_at
     FROM images
     WHERE last_used_at IS NOT NULL
     ORDER BY last_used_at DESC, id DESC
     LIMIT ?1",
  )?;

  let rows = statement.query_map(params![limit], map_gallery_row)?;

  let mut items = Vec::new();
  for row in rows {
    let mut image = row?;
    image.thumb_file_path = image
      .thumb_path
      .as_ref()
      .map(|relative| state.app_data_dir.join(relative).to_string_lossy().to_string());
    items.push(image);
  }

  Ok(items)
}

fn run_get_image_path(state: &AppState, image_id: i64) -> Result<String, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  resolve_image_path(&connection, image_id)
}

fn run_mark_image_used(state: &AppState, image_id: i64) -> Result<(), AppError> {
  let connection = db::open_connection(&state.database_path)?;
  mark_image_used_in_db(&connection, image_id)
}

fn run_copy_image_to_clipboard(state: &AppState, image_id: i64) -> Result<(), AppError> {
  let connection = db::open_connection(&state.database_path)?;
  let path = resolve_image_path(&connection, image_id)?;
  clipboard::copy_image_file_to_clipboard(std::path::Path::new(&path))?;
  mark_image_used_in_db(&connection, image_id)
}

fn run_get_tags(
  state: &AppState,
  query: Option<&str>,
  limit: i64,
) -> Result<Vec<TagItem>, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  tags::list_tags(&connection, query, limit)
}

fn run_get_image_tags(state: &AppState, image_id: i64) -> Result<Vec<TagItem>, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  tags::get_tags_for_image(&connection, image_id)
}

fn run_create_tag(
  state: &AppState,
  name: &str,
  color: Option<String>,
  parent_id: Option<i64>,
) -> Result<TagItem, AppError> {
  let connection = db::open_connection(&state.database_path)?;
  tags::upsert_tag(&connection, name, color, parent_id)
}

fn run_delete_tag(state: &AppState, tag_id: i64) -> Result<(), AppError> {
  let connection = db::open_connection(&state.database_path)?;
  tags::delete_tag(&connection, tag_id)
}

fn run_add_tags_to_images(
  state: &AppState,
  image_ids: &[i64],
  tag_names: &[String],
) -> Result<TagMutationResult, AppError> {
  let mut connection = db::open_connection(&state.database_path)?;
  tags::add_tags_to_images(&mut connection, image_ids, tag_names)
}

fn run_remove_tags_from_images(
  state: &AppState,
  image_ids: &[i64],
  tag_names: &[String],
) -> Result<TagMutationResult, AppError> {
  let mut connection = db::open_connection(&state.database_path)?;
  tags::remove_tags_from_images(&mut connection, image_ids, tag_names)
}

enum ImportOutcome {
  Imported,
  Skipped,
}

fn import_single_image(
  file: &ScannedFile,
  app_data_dir: &std::path::Path,
  existing_hash_stmt: &mut rusqlite::Statement<'_>,
  insert_stmt: &mut rusqlite::Statement<'_>,
) -> Result<ImportOutcome, AppError> {
  let hash = scanner::compute_sha256(&file.path)?;

  let is_duplicate = existing_hash_stmt
    .query_row(params![&hash], |_| Ok(()))
    .optional()?
    .is_some();

  if is_duplicate {
    return Ok(ImportOutcome::Skipped);
  }

  let (width, height) =
    ::image::image_dimensions(&file.path).map_err(|source| AppError::ImageProcess {
      path: file.path.clone(),
      source,
    })?;

  let thumbnail = thumbnail::create_thumbnail(&file.path, app_data_dir, &hash)?;
  let filename = file
    .path
    .file_name()
    .and_then(|name| name.to_str())
    .unwrap_or_default()
    .to_string();
  let mime_type = crate::image::guess_mime_type(&file.extension)
    .unwrap_or("application/octet-stream")
    .to_string();

  insert_stmt.execute(params![
    filename,
    file.path.to_string_lossy().to_string(),
    hash,
    i64::from(width),
    i64::from(height),
    file.file_size,
    mime_type,
    thumbnail.relative_path,
  ])?;

  Ok(ImportOutcome::Imported)
}

fn resolve_image_path(connection: &rusqlite::Connection, image_id: i64) -> Result<String, AppError> {
  let path = connection
    .query_row(
      "SELECT filepath FROM images WHERE id = ?1",
      params![image_id],
      |row| row.get(0),
    )
    .optional()?;

  path.ok_or_else(|| AppError::NotFound(format!("image id {image_id} not found")))
}

fn mark_image_used_in_db(connection: &rusqlite::Connection, image_id: i64) -> Result<(), AppError> {
  let changed = connection.execute(
    "UPDATE images SET last_used_at = datetime('now') WHERE id = ?1",
    params![image_id],
  )?;

  if changed == 0 {
    return Err(AppError::NotFound(format!("image id {image_id} not found")));
  }

  Ok(())
}

fn map_gallery_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GalleryImage> {
  Ok(GalleryImage {
    id: row.get(0)?,
    filename: row.get(1)?,
    filepath: row.get(2)?,
    thumb_path: row.get(3)?,
    thumb_file_path: None,
    width: row.get(4)?,
    height: row.get(5)?,
    file_size: row.get(6)?,
    mime_type: row.get(7)?,
    imported_at: row.get(8)?,
    last_used_at: row.get(9)?,
  })
}