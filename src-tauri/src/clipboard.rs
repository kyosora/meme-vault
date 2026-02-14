use std::borrow::Cow;
use std::path::Path;

use arboard::{Clipboard, ImageData};

use crate::error::AppError;

pub fn copy_image_file_to_clipboard(path: &Path) -> Result<(), AppError> {
  let image = image::open(path).map_err(|source| AppError::ImageProcess {
    path: path.to_path_buf(),
    source,
  })?;

  let rgba = image.to_rgba8();
  let (width, height) = rgba.dimensions();

  let data = ImageData {
    width: width as usize,
    height: height as usize,
    bytes: Cow::Owned(rgba.into_raw()),
  };

  let mut clipboard = Clipboard::new()?;
  clipboard.set_image(data)?;

  Ok(())
}