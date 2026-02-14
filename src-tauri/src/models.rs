use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
  pub app_data_dir: String,
  pub database_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
  pub ok: bool,
  pub image_count: i64,
  pub tag_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryImage {
  pub id: i64,
  pub filename: String,
  pub filepath: String,
  pub thumb_path: Option<String>,
  pub thumb_file_path: Option<String>,
  pub width: Option<i64>,
  pub height: Option<i64>,
  pub file_size: i64,
  pub mime_type: String,
  pub imported_at: String,
  pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagesPage {
  pub items: Vec<GalleryImage>,
  pub total: i64,
  pub page: i64,
  pub limit: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
  pub current: usize,
  pub total: usize,
  pub imported: usize,
  pub skipped: usize,
  pub failed: usize,
  pub current_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
  pub imported: usize,
  pub skipped: usize,
  pub failed: usize,
  pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagItem {
  pub id: i64,
  pub name: String,
  pub parent_id: Option<i64>,
  pub color: Option<String>,
  pub usage_count: i64,
  pub image_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagMutationResult {
  pub selected_images: usize,
  pub resolved_tags: usize,
  pub link_changes: usize,
}