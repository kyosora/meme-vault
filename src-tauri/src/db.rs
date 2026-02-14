use std::path::Path;

use rusqlite::Connection;

use crate::error::AppError;

pub const DATABASE_FILE_NAME: &str = "meme-vault.db";

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL UNIQUE,
    hash TEXT NOT NULL UNIQUE,
    width INTEGER,
    height INTEGER,
    file_size INTEGER NOT NULL,
    mime_type TEXT NOT NULL,
    thumb_path TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    imported_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    parent_id INTEGER REFERENCES tags(id) ON DELETE SET NULL,
    color TEXT,
    usage_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS image_tags (
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    tagged_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (image_id, tag_id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS tags_fts USING fts5(
    name,
    content='tags',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS tags_ai AFTER INSERT ON tags BEGIN
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

CREATE TRIGGER IF NOT EXISTS tags_ad AFTER DELETE ON tags BEGIN
    INSERT INTO tags_fts(tags_fts, rowid, name) VALUES ('delete', old.id, old.name);
END;

CREATE TRIGGER IF NOT EXISTS tags_au AFTER UPDATE OF name ON tags BEGIN
    INSERT INTO tags_fts(tags_fts, rowid, name) VALUES ('delete', old.id, old.name);
    INSERT INTO tags_fts(rowid, name) VALUES (new.id, new.name);
END;

CREATE TRIGGER IF NOT EXISTS image_tags_insert AFTER INSERT ON image_tags BEGIN
    UPDATE tags SET usage_count = usage_count + 1 WHERE id = new.tag_id;
END;

CREATE TRIGGER IF NOT EXISTS image_tags_delete AFTER DELETE ON image_tags BEGIN
    UPDATE tags SET usage_count = usage_count - 1 WHERE id = old.tag_id;
END;

CREATE INDEX IF NOT EXISTS idx_images_hash ON images(hash);
CREATE INDEX IF NOT EXISTS idx_images_imported ON images(imported_at);
CREATE INDEX IF NOT EXISTS idx_images_last_used ON images(last_used_at);
CREATE INDEX IF NOT EXISTS idx_image_tags_tag ON image_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_image_tags_image ON image_tags(image_id);
CREATE INDEX IF NOT EXISTS idx_tags_parent ON tags(parent_id);
CREATE INDEX IF NOT EXISTS idx_tags_usage ON tags(usage_count DESC);
"#;

pub fn initialize_database(database_path: &Path) -> Result<(), AppError> {
    let connection = Connection::open(database_path)?;
    configure_connection(&connection)?;
    connection.execute_batch(SCHEMA_SQL)?;
    Ok(())
}

pub fn open_connection(database_path: &Path) -> Result<Connection, AppError> {
    let connection = Connection::open(database_path)?;
    configure_connection(&connection)?;
    Ok(connection)
}

fn configure_connection(connection: &Connection) -> Result<(), AppError> {
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}
