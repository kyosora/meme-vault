use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::error::AppError;
use crate::image;

#[derive(Debug, Clone)]
pub struct ScannedFile {
  pub path: PathBuf,
  pub extension: String,
  pub file_size: i64,
}

pub fn collect_images(root: &Path) -> Result<Vec<ScannedFile>, AppError> {
  if !root.exists() {
    return Err(AppError::InvalidInput(format!(
      "import path does not exist: {}",
      root.display()
    )));
  }

  if !root.is_dir() {
    return Err(AppError::InvalidInput(format!(
      "import path is not a directory: {}",
      root.display()
    )));
  }

  let mut images = Vec::new();

  for entry in WalkDir::new(root).follow_links(true) {
    let entry = entry?;
    if !entry.file_type().is_file() {
      continue;
    }

    let path = entry.path();
    let Some(extension) = path.extension().and_then(OsStr::to_str) else {
      continue;
    };

    if !image::is_supported_extension(extension) {
      continue;
    }

    let metadata = path.metadata().map_err(|source| AppError::FileRead {
      path: path.to_path_buf(),
      source,
    })?;

    let file_size = i64::try_from(metadata.len())
      .map_err(|_| AppError::InvalidInput(format!("file too large: {}", path.display())))?;

    images.push(ScannedFile {
      path: path.to_path_buf(),
      extension: extension.to_ascii_lowercase(),
      file_size,
    });
  }

  images.sort_by(|left, right| left.path.cmp(&right.path));

  Ok(images)
}

pub fn compute_sha256(path: &Path) -> Result<String, AppError> {
  let mut file = File::open(path).map_err(|source| AppError::FileRead {
    path: path.to_path_buf(),
    source,
  })?;

  let mut hasher = Sha256::new();
  let mut buffer = [0_u8; 8192];

  loop {
    let read_count = file.read(&mut buffer).map_err(|source| AppError::FileRead {
      path: path.to_path_buf(),
      source,
    })?;

    if read_count == 0 {
      break;
    }

    hasher.update(&buffer[..read_count]);
  }

  Ok(format!("{:x}", hasher.finalize()))
}