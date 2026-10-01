use rusqlite::{params, Connection, Result, OptionalExtension};
use parking_lot::Mutex;
use serde_json::Value;
use std::sync::Arc;

pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
}

impl DatabaseManager {
    pub fn new(db_path: &str) -> Result<Self> {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let conn = Connection::open(db_path)?;
        
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             CREATE TABLE IF NOT EXISTS config (key TEXT PRIMARY KEY, value TEXT);
             CREATE TABLE IF NOT EXISTS mod_cache (sha1 TEXT PRIMARY KEY, metadata TEXT, timestamp DATETIME DEFAULT CURRENT_TIMESTAMP);
             CREATE TABLE IF NOT EXISTS play_time (instance TEXT PRIMARY KEY, seconds INTEGER DEFAULT 0);
             CREATE TABLE IF NOT EXISTS friends (name TEXT PRIMARY KEY, added_at DATETIME DEFAULT CURRENT_TIMESTAMP);"
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get_config(&self, key: &str) -> Option<Value> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?").ok()?;
        let result: Option<String> = stmt.query_row(params![key], |row| row.get(0)).optional().unwrap_or(None);
        result.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn set_config(&self, key: &str, value: &Value) {
        let conn = self.conn.lock();
        let val_str = serde_json::to_string(value).unwrap_or_default();
        let _ = conn.execute("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)", params![key, val_str]);
    }

    pub fn get_mod_meta(&self, sha1: &str) -> Option<Value> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT metadata FROM mod_cache WHERE sha1 = ?").ok()?;
        let result: Option<String> = stmt.query_row(params![sha1], |row| row.get(0)).optional().unwrap_or(None);
        result.and_then(|v| serde_json::from_str(&v).ok())
    }

    pub fn set_mod_meta(&self, sha1: &str, metadata: &Value) {
        let conn = self.conn.lock();
        let meta_str = serde_json::to_string(metadata).unwrap_or_default();
        let _ = conn.execute("INSERT OR REPLACE INTO mod_cache (sha1, metadata) VALUES (?, ?)", params![sha1, meta_str]);
    }

    pub fn add_play_time(&self, instance: &str, seconds: i64) {
        let conn = self.conn.lock();
        let query = "INSERT INTO play_time (instance, seconds) VALUES (?1, ?2) ON CONFLICT(instance) DO UPDATE SET seconds = seconds + ?2";
        let _ = conn.execute(query, params![instance, seconds]);
    }

    pub fn get_play_time(&self, instance: &str) -> i64 {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare("SELECT seconds FROM play_time WHERE instance = ?") {
            Ok(s) => s,
            Err(_) => return 0,
        };
        stmt.query_row(params![instance], |row| row.get(0)).unwrap_or(0)
    }

    pub fn get_friends(&self) -> Vec<String> {
        let conn = self.conn.lock();
        let mut stmt = match conn.prepare("SELECT name FROM friends ORDER BY added_at DESC") {
            Ok(s) => s,
            Err(_) => return Vec::new(),
        };
        let names = match stmt.query_map([], |row| row.get(0)) {
            Ok(rows) => rows.filter_map(|r| r.ok()).collect(),
            Err(_) => Vec::new(),
        };
        names
    }

    pub fn add_friend(&self, name: &str) -> bool {
        let conn = self.conn.lock();
        conn.execute("INSERT INTO friends (name) VALUES (?)", params![name]).is_ok()
    }

    pub fn remove_friend(&self, name: &str) -> bool {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM friends WHERE name = ?", params![name]).map(|count| count > 0).unwrap_or(false)
    }
}