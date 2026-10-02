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
    pub added_at: String,
}

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
                 added_at DATETIME DEFAULT CURRENT_TIMESTAMP
             );
             CREATE INDEX IF NOT EXISTS idx_friends_added ON friends(added_at DESC);",
        )?;

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

    pub fn get_friends(&self) -> Vec<String> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare_cached("SELECT name FROM friends ORDER BY added_at DESC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let names = match stmt.query_map([], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };
        names
    }

    pub fn add_friend(&self, name: &str) -> bool {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return false;
        }
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR IGNORE INTO friends (name, added_at) VALUES (?1, CURRENT_TIMESTAMP)",
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

    pub fn checkpoint(&self) -> bool {
        let conn = self.conn.lock();
        conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);").is_ok()
    }
}