use std::path::{PathBuf, StripPrefixError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("failed to resolve app data directory: {0}")]
  PathResolve(#[source] tauri::Error),

  #[error("failed to create app data directory '{path}': {source}")]
  DirectoryCreate {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("database error: {0}")]
  Database(#[from] rusqlite::Error),

  #[error("failed to read file '{path}': {source}")]
  FileRead {
    path: PathBuf,
    #[source]
    source: std::io::Error,
  },

  #[error("failed to process image '{path}': {source}")]
  ImageProcess {
    path: PathBuf,
    #[source]
    source: image::ImageError,
  },

  #[error("clipboard error: {0}")]
  Clipboard(#[from] arboard::Error),

  #[error("failed to scan filesystem: {0}")]
  Walkdir(#[from] walkdir::Error),

  #[error("failed to emit event: {0}")]
  Event(#[from] tauri::Error),

  #[error("path '{path}' is not under base '{base}': {source}")]
  PathStripPrefix {
    path: PathBuf,
    base: PathBuf,
    #[source]
    source: StripPrefixError,
  },

  #[error("not found: {0}")]
  NotFound(String),

  #[error("invalid input: {0}")]
  InvalidInput(String),
}