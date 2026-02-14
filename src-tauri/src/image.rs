pub const SUPPORTED_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

#[allow(dead_code)]
pub fn is_supported_extension(extension: &str) -> bool {
  SUPPORTED_EXTENSIONS
    .iter()
    .any(|item| item.eq_ignore_ascii_case(extension))
}

#[allow(dead_code)]
pub fn guess_mime_type(extension: &str) -> Option<&'static str> {
  let normalized = extension.to_ascii_lowercase();
  match normalized.as_str() {
    "png" => Some("image/png"),
    "jpg" | "jpeg" => Some("image/jpeg"),
    "gif" => Some("image/gif"),
    "webp" => Some("image/webp"),
    _ => None,
  }
}
