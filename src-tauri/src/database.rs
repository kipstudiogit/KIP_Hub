use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FriendRecordEntity {
    pub name: String,
    pub is_favorite: bool,
    pub note: String,
    pub added_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaytimeRecordEntity {
    pub instance: String,
    pub seconds: i64,
}

pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new(db_path: &str) -> Result<Self> {
        if let Some(parent) = Path::new(db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(db_path)?;
        conn.busy_timeout(Duration::from_millis(3000))?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -8000;
             CREATE TABLE IF NOT EXISTS config (
                 key TEXT PRIMARY KEY,
                 value TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS mod_cache (
                 sha1 TEXT PRIMARY KEY,
                 metadata TEXT NOT NULL,
                 timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
             );
             CREATE TABLE IF NOT EXISTS play_time (
                 instance TEXT PRIMARY KEY,
                 seconds INTEGER NOT NULL DEFAULT 0
             );
             CREATE TABLE IF NOT EXISTS friends (
                 name TEXT PRIMARY KEY,
                 is_favorite INTEGER NOT NULL DEFAULT 0,
                 note TEXT DEFAULT '',
                 added_at DATETIME DEFAULT CURRENT_TIMESTAMP
             );
             CREATE INDEX IF NOT EXISTS idx_friends_added ON friends(is_favorite DESC, added_at DESC);",
        )?;

        let _ = conn.execute_batch(
            "ALTER TABLE friends ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
             ALTER TABLE friends ADD COLUMN note TEXT DEFAULT '';"
        );

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_config(&self, key: &str) -> Option<Value> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare_cached("SELECT value FROM config WHERE key = ?1").ok()?;
        let result: Option<String> = stmt
            .query_row(params![key], |row| row.get(0))
            .optional()
            .unwrap_or(None);
        result.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn set_config(&self, key: &str, value: &Value) -> bool {
        let conn = self.conn.lock();
        let val_str = serde_json::to_string(value).unwrap_or_default();
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, val_str],
        )
        .is_ok()
    }

    pub fn get_mod_meta(&self, sha1: &str) -> Option<Value> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare_cached("SELECT metadata FROM mod_cache WHERE sha1 = ?1").ok()?;
        let result: Option<String> = stmt
            .query_row(params![sha1], |row| row.get(0))
            .optional()
            .unwrap_or(None);
        result.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn set_mod_meta(&self, sha1: &str, metadata: &Value) -> bool {
        let conn = self.conn.lock();
        let meta_str = serde_json::to_string(metadata).unwrap_or_default();
        conn.execute(
            "INSERT INTO mod_cache (sha1, metadata, timestamp) VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(sha1) DO UPDATE SET metadata = excluded.metadata, timestamp = CURRENT_TIMESTAMP",
            params![sha1, meta_str],
        )
        .is_ok()
    }

    pub fn add_play_time(&self, instance: &str, seconds: i64) -> bool {
        if seconds <= 0 || instance.trim().is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        let query = "INSERT INTO play_time (instance, seconds) VALUES (?1, ?2)
                     ON CONFLICT(instance) DO UPDATE SET seconds = play_time.seconds + excluded.seconds";
        conn.execute(query, params![instance, seconds]).is_ok()
    }

    pub fn get_play_time(&self, instance: &str) -> i64 {
        if instance.trim().is_empty() {
            return 0;
        }
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare_cached("SELECT seconds FROM play_time WHERE instance = ?1") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        stmt.query_row(params![instance], |row| row.get(0)).unwrap_or(0)
    }

    pub fn get_all_playtimes(&self) -> Vec<PlaytimeRecordEntity> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare_cached("SELECT instance, seconds FROM play_time ORDER BY seconds DESC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map([], |row| {
            Ok(PlaytimeRecordEntity {
                instance: row.get(0)?,
                seconds: row.get(1)?,
            })
        }) {
            Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };

        rows
    }

    pub fn get_friends(&self) -> Vec<FriendRecordEntity> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare_cached("SELECT name, is_favorite, note, added_at FROM friends ORDER BY is_favorite DESC, added_at DESC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };

        let rows = match stmt.query_map([], |row| {
            let fav_num: i32 = row.get(1).unwrap_or(0);
            Ok(FriendRecordEntity {
                name: row.get(0)?,
                is_favorite: fav_num == 1,
                note: row.get(2).unwrap_or_default(),
                added_at: row.get(3).unwrap_or_default(),
            })
        }) {
            Ok(mapped) => mapped.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };

        rows
    }

    pub fn add_friend(&self, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO friends (name, is_favorite, note, added_at) VALUES (?1, 0, '', CURRENT_TIMESTAMP)",
            params![trimmed],
        )
        .map(|count| count > 0)
        .unwrap_or(false)
    }

    pub fn remove_friend(&self, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        conn.execute("DELETE FROM friends WHERE name = ?1", params![trimmed])
            .map(|count| count > 0)
            .unwrap_or(false)
    }

    pub fn toggle_favorite_friend(&self, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE friends SET is_favorite = CASE WHEN is_favorite = 1 THEN 0 ELSE 1 END WHERE name = ?1",
            params![trimmed],
        )
        .map(|count| count > 0)
        .unwrap_or(false)
    }

    pub fn update_friend_note(&self, name: &str, note: &str) -> bool {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        conn.execute(
            "UPDATE friends SET note = ?2 WHERE name = ?1",
            params![trimmed_name, note.trim()],
        )
        .map(|count| count > 0)
        .unwrap_or(false)
    }

    pub fn checkpoint(&self) -> bool {
        let conn = self.conn.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);").is_ok()
    }
}