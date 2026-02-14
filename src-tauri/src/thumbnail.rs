use std::fs;
use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};

use crate::error::AppError;

const THUMB_EDGE: u32 = 200;
const THUMB_DIRECTORY: &str = "thumbs";

#[derive(Debug, Clone)]
pub struct ThumbnailOutput {
  pub relative_path: String,
}

pub fn create_thumbnail(
  source_path: &Path,
  app_data_dir: &Path,
  hash: &str,
) -> Result<ThumbnailOutput, AppError> {
  let source_image = image::open(source_path).map_err(|source| AppError::ImageProcess {
    path: source_path.to_path_buf(),
    source,
  })?;

  let thumbnail_image = create_square_thumbnail(source_image);

  let bucket = hash
    .get(..2)
    .ok_or_else(|| AppError::InvalidInput("hash is shorter than two characters".to_string()))?;

  let directory = app_data_dir.join(THUMB_DIRECTORY).join(bucket);
  fs::create_dir_all(&directory).map_err(|source| AppError::DirectoryCreate {
    path: directory.clone(),
    source,
  })?;

  let absolute_path = directory.join(format!("{hash}.webp"));
  if !absolute_path.exists() {
    thumbnail_image
      .save_with_format(&absolute_path, ImageFormat::WebP)
      .map_err(|source| AppError::ImageProcess {
        path: absolute_path.clone(),
        source,
      })?;
  }

  let relative = absolute_path
    .strip_prefix(app_data_dir)
    .map_err(|source| AppError::PathStripPrefix {
      path: absolute_path.clone(),
      base: app_data_dir.to_path_buf(),
      source,
    })?
    .to_string_lossy()
    .replace('\\', "/");

  Ok(ThumbnailOutput {
    relative_path: relative,
  })
}

fn create_square_thumbnail(image: DynamicImage) -> DynamicImage {
  let (width, height) = image.dimensions();
  let edge = width.min(height);
  let left = (width - edge) / 2;
  let top = (height - edge) / 2;

  let cropped = image.crop_imm(left, top, edge, edge);
  cropped.resize_exact(THUMB_EDGE, THUMB_EDGE, FilterType::CatmullRom)
}