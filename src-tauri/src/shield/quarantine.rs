use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineRecordDto {
    pub id: String,
    pub original_path: String,
    pub filename: String,
    pub isolated_filename: String,
    pub threat_name: String,
    pub threat_score: u32,
    pub sha256: String,
    pub quarantined_at: String,
    pub size_bytes: u64,
}

pub struct QuarantineVault {
    vault_dir: PathBuf,
    manifest_path: PathBuf,
}

impl QuarantineVault {
    pub fn new() -> Self {
        let vault_dir = config::get_app_data_dir().join("shield_quarantine");
        let _ = fs::create_dir_all(&vault_dir);
        let manifest_path = vault_dir.join("vault_manifest.json");

        Self {
            vault_dir,
            manifest_path,
        }
    }

    fn xor_mask(data: &mut [u8]) {
        let key = b"KIP_SHIELD_VAULT_ISOLATION_KEY_9981";
        for (i, byte) in data.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
    }

    pub fn load_records(&self) -> Vec<QuarantineRecordDto> {
        if self.manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.manifest_path) {
                if let Ok(list) = serde_json::from_str::<Vec<QuarantineRecordDto>>(&content) {
                    return list;
                }
            }
        }
        Vec::new()
    }

    fn save_records(&self, records: &[QuarantineRecordDto]) -> bool {
        if let Ok(serialized) = serde_json::to_string_pretty(records) {
            return fs::write(&self.manifest_path, serialized).is_ok();
        }
        false
    }

    pub fn quarantine_file(
        &self,
        src_path: &Path,
        threat_name: &str,
        threat_score: u32,
        sha256: &str,
    ) -> Result<QuarantineRecordDto, String> {
        if !src_path.exists() || !src_path.is_file() {
            return Err("Target threat file does not exist on disk.".to_string());
        }

        let mut buffer = Vec::new();
        let mut file = File::open(src_path).map_err(|e| e.to_string())?;
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        drop(file);

        let size_bytes = buffer.len() as u64;
        Self::xor_mask(&mut buffer);

        let id = Uuid::new_v4().to_string();
        let isolated_filename = format!("{}.kip_quarantine", id);
        let dest_path = self.vault_dir.join(&isolated_filename);

        fs::write(&dest_path, buffer).map_err(|e| e.to_string())?;
        fs::remove_file(src_path).map_err(|e| e.to_string())?;

        let record = QuarantineRecordDto {
            id,
            original_path: src_path.to_string_lossy().to_string(),
            filename: src_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            isolated_filename,
            threat_name: threat_name.to_string(),
            threat_score,
            sha256: sha256.to_string(),
            quarantined_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            size_bytes,
        };

        let mut records = self.load_records();
        records.retain(|r| r.original_path != record.original_path);
        records.push(record.clone());
        self.save_records(&records);

        Ok(record)
    }

    pub fn restore_file(&self, id: &str) -> Result<bool, String> {
        let mut records = self.load_records();
        let index = records.iter().position(|r| r.id == id).ok_or("Quarantine record not found.")?;
        let record = records.remove(index);

        let isolated_path = self.vault_dir.join(&record.isolated_filename);
        if !isolated_path.exists() {
            return Err("Isolated vault archive payload missing.".to_string());
        }

        let mut buffer = Vec::new();
        let mut file = File::open(&isolated_path).map_err(|e| e.to_string())?;
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
        drop(file);

        Self::xor_mask(&mut buffer);

        let target_path = PathBuf::from(&record.original_path);
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        fs::write(&target_path, buffer).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&isolated_path);
        self.save_records(&records);

        Ok(true)
    }

    pub fn shred_file(&self, id: &str) -> Result<bool, String> {
        let mut records = self.load_records();
        let index = records.iter().position(|r| r.id == id).ok_or("Quarantine record not found.")?;
        let record = records.remove(index);

        let isolated_path = self.vault_dir.join(&record.isolated_filename);
        if isolated_path.exists() {
            if let Ok(meta) = fs::metadata(&isolated_path) {
                let len = meta.len() as usize;
                let zeroes = vec![0u8; len];
                if let Ok(mut f) = OpenOptions::new().write(true).open(&isolated_path) {
                    let _ = f.write_all(&zeroes);
                    let _ = f.sync_all();
                }
            }
            let _ = fs::remove_file(&isolated_path);
        }

        self.save_records(&records);
        Ok(true)
    }
}