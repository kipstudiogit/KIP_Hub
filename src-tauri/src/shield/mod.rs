pub mod analyzer;
pub mod quarantine;
pub mod signatures;

use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::shield::analyzer::{FileSecurityReportDto, StaticBytecodeAnalyzer};
use crate::shield::quarantine::{QuarantineRecordDto, QuarantineVault};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldScanReportDto {
    pub total_scanned: usize,
    pub clean_count: usize,
    pub threat_count: usize,
    pub critical_count: usize,
    pub threats: Vec<FileSecurityReportDto>,
    pub timestamp: String,
}

pub struct ShieldManager {
    analyzer: StaticBytecodeAnalyzer,
    vault: Arc<QuarantineVault>,
}

impl ShieldManager {
    pub fn new() -> Self {
        Self {
            analyzer: StaticBytecodeAnalyzer::new(),
            vault: Arc::new(QuarantineVault::new()),
        }
    }

    #[allow(dead_code)]
    pub fn scan_single_file(&self, filepath: &str) -> FileSecurityReportDto {
        self.analyzer.analyze_file(Path::new(filepath))
    }

    pub fn scan_directory(&self, dir_path: &str) -> ShieldScanReportDto {
        let path = Path::new(dir_path);
        let mut total_scanned = 0;
        let mut clean_count = 0;
        let mut critical_count = 0;
        let mut threats = Vec::new();

        if path.exists() {
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    if ext == "jar" || ext == "zip" {
                        total_scanned += 1;
                        let report = self.analyzer.analyze_file(p);
                        if report.is_clean {
                            clean_count += 1;
                        } else {
                            if report.threat_level == "CRITICAL" {
                                critical_count += 1;
                            }
                            threats.push(report);
                        }
                    }
                }
            }
        }

        threats.sort_by(|a, b| b.threat_score.cmp(&a.threat_score));
        let threat_count = threats.len();

        ShieldScanReportDto {
            total_scanned,
            clean_count,
            threat_count,
            critical_count,
            threats,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    pub fn quarantine(&self, filepath: &str) -> Result<QuarantineRecordDto, String> {
        let p = Path::new(filepath);
        let report = self.analyzer.analyze_file(p);
        let threat_name = report
            .indicators
            .first()
            .map(|i| i.title.clone())
            .unwrap_or_else(|| "Generic Suspicious Signature".to_string());
        self.vault
            .quarantine_file(p, &threat_name, report.threat_score, &report.sha256)
    }

    pub fn restore(&self, id: &str) -> Result<bool, String> {
        self.vault.restore_file(id)
    }

    pub fn shred(&self, id: &str) -> Result<bool, String> {
        self.vault.shred_file(id)
    }

    pub fn get_vault_records(&self) -> Vec<QuarantineRecordDto> {
        self.vault.load_records()
    }
}