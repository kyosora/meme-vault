use std::collections::BTreeSet;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::AppError;
use crate::models::{TagItem, TagMutationResult};

const TAG_COLOR_PALETTE: [&str; 10] = [
  "#f97316",
  "#22c55e",
  "#06b6d4",
  "#0ea5e9",
  "#a855f7",
  "#ec4899",
  "#eab308",
  "#ef4444",
  "#6366f1",
  "#14b8a6",
];

pub fn list_tags(
  connection: &Connection,
  query: Option<&str>,
  limit: i64,
) -> Result<Vec<TagItem>, AppError> {
  let resolved_limit = limit.clamp(1, 200);
  let trimmed = query.unwrap_or("").trim();

  let sql = format!(
    "SELECT
        t.id,
        t.name,
        t.parent_id,
        t.color,
        t.usage_count,
        COUNT(it.image_id) AS image_count
      FROM tags t
      LEFT JOIN image_tags it ON it.tag_id = t.id
      {}
      GROUP BY t.id
      ORDER BY t.usage_count DESC, t.name ASC
      LIMIT {resolved_limit}",
    if trimmed.is_empty() {
      String::new()
    } else {
      "WHERE t.name LIKE '%' || ?1 || '%' COLLATE NOCASE".to_string()
    }
  );

  let mut statement = connection.prepare(&sql)?;
  let rows = if trimmed.is_empty() {
    statement.query_map([], map_tag_row)?
  } else {
    statement.query_map(params![trimmed], map_tag_row)?
  };

  let mut tags = Vec::new();
  for row in rows {
    tags.push(row?);
  }

  Ok(tags)
}

pub fn get_tags_for_image(connection: &Connection, image_id: i64) -> Result<Vec<TagItem>, AppError> {
  let mut statement = connection.prepare(
    "SELECT
        t.id,
        t.name,
        t.parent_id,
        t.color,
        t.usage_count,
        COUNT(it_all.image_id) AS image_count
      FROM tags t
      JOIN image_tags it_current ON it_current.tag_id = t.id
      LEFT JOIN image_tags it_all ON it_all.tag_id = t.id
      WHERE it_current.image_id = ?1
      GROUP BY t.id
      ORDER BY t.name ASC",
  )?;

  let rows = statement.query_map(params![image_id], map_tag_row)?;

  let mut tags = Vec::new();
  for row in rows {
    tags.push(row?);
  }

  Ok(tags)
}

pub fn upsert_tag(
  connection: &Connection,
  name: &str,
  color: Option<String>,
  parent_id: Option<i64>,
) -> Result<TagItem, AppError> {
  let trimmed = name.trim();
  if trimmed.is_empty() {
    return Err(AppError::InvalidInput("tag name cannot be empty".to_string()));
  }

  let fallback_color = default_color_for_name(trimmed);

  connection.execute(
    "INSERT OR IGNORE INTO tags (name, color, parent_id) VALUES (?1, COALESCE(?2, ?4), ?3)",
    params![trimmed, color, parent_id, fallback_color],
  )?;

  connection.execute(
    "UPDATE tags
      SET color = COALESCE(?2, color),
          parent_id = COALESCE(?3, parent_id)
      WHERE name = ?1",
    params![trimmed, color, parent_id],
  )?;

  let mut statement = connection.prepare(
    "SELECT
        t.id,
        t.name,
        t.parent_id,
        t.color,
        t.usage_count,
        COUNT(it.image_id) AS image_count
      FROM tags t
      LEFT JOIN image_tags it ON it.tag_id = t.id
      WHERE t.name = ?1
      GROUP BY t.id
      LIMIT 1",
  )?;

  let tag = statement.query_row(params![trimmed], map_tag_row)?;
  Ok(tag)
}

pub fn delete_tag(connection: &Connection, tag_id: i64) -> Result<(), AppError> {
  connection.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
  Ok(())
}

pub fn add_tags_to_images(
  connection: &mut Connection,
  image_ids: &[i64],
  tag_names: &[String],
) -> Result<TagMutationResult, AppError> {
  let normalized_ids = normalize_image_ids(image_ids);
  let normalized_tag_names = normalize_tag_names(tag_names);

  if normalized_ids.is_empty() || normalized_tag_names.is_empty() {
    return Ok(TagMutationResult {
      selected_images: normalized_ids.len(),
      resolved_tags: normalized_tag_names.len(),
      link_changes: 0,
    });
  }

  let tx = connection.transaction()?;

  let mut tag_ids: Vec<i64> = Vec::new();
  for tag_name in &normalized_tag_names {
    let fallback_color = default_color_for_name(tag_name);
    tx.execute(
      "INSERT OR IGNORE INTO tags (name, color) VALUES (?1, ?2)",
      params![tag_name, fallback_color],
    )?;

    let tag_id: i64 = tx.query_row(
      "SELECT id FROM tags WHERE name = ?1 LIMIT 1",
      params![tag_name],
      |row| row.get(0),
    )?;
    tag_ids.push(tag_id);
  }

  let mut link_changes = 0_usize;
  for image_id in &normalized_ids {
    for tag_id in &tag_ids {
      let changed = tx.execute(
        "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
        params![image_id, tag_id],
      )?;
      link_changes += changed;
    }
  }

  tx.commit()?;

  Ok(TagMutationResult {
    selected_images: normalized_ids.len(),
    resolved_tags: tag_ids.len(),
    link_changes,
  })
}

pub fn remove_tags_from_images(
  connection: &mut Connection,
  image_ids: &[i64],
  tag_names: &[String],
) -> Result<TagMutationResult, AppError> {
  let normalized_ids = normalize_image_ids(image_ids);
  let normalized_tag_names = normalize_tag_names(tag_names);

  if normalized_ids.is_empty() || normalized_tag_names.is_empty() {
    return Ok(TagMutationResult {
      selected_images: normalized_ids.len(),
      resolved_tags: normalized_tag_names.len(),
      link_changes: 0,
    });
  }

  let tx = connection.transaction()?;

  let mut tag_ids: Vec<i64> = Vec::new();
  for tag_name in &normalized_tag_names {
    let maybe_id: Option<i64> = tx
      .query_row(
        "SELECT id FROM tags WHERE name = ?1 LIMIT 1",
        params![tag_name],
        |row| row.get(0),
      )
      .optional()?;

    if let Some(tag_id) = maybe_id {
      tag_ids.push(tag_id);
    }
  }

  let mut link_changes = 0_usize;
  for image_id in &normalized_ids {
    for tag_id in &tag_ids {
      let changed = tx.execute(
        "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2",
        params![image_id, tag_id],
      )?;
      link_changes += changed;
    }
  }

  tx.commit()?;

  Ok(TagMutationResult {
    selected_images: normalized_ids.len(),
    resolved_tags: tag_ids.len(),
    link_changes,
  })
}

fn normalize_image_ids(image_ids: &[i64]) -> Vec<i64> {
  image_ids
    .iter()
    .copied()
    .filter(|id| *id > 0)
    .collect::<BTreeSet<i64>>()
    .into_iter()
    .collect()
}

fn normalize_tag_names(tag_names: &[String]) -> Vec<String> {
  tag_names
    .iter()
    .map(|name| name.trim())
    .filter(|name| !name.is_empty())
    .map(|name| name.to_string())
    .collect::<BTreeSet<String>>()
    .into_iter()
    .collect()
}

fn default_color_for_name(name: &str) -> String {
  let mut hash: usize = 5381;
  for byte in name.as_bytes() {
    hash = ((hash << 5).wrapping_add(hash)).wrapping_add(*byte as usize);
  }

  TAG_COLOR_PALETTE[hash % TAG_COLOR_PALETTE.len()].to_string()
}

fn map_tag_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TagItem> {
  Ok(TagItem {
    id: row.get(0)?,
    name: row.get(1)?,
    parent_id: row.get(2)?,
    color: row.get(3)?,
    usage_count: row.get(4)?,
    image_count: row.get(5)?,
  })
}