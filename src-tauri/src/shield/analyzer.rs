use std::fs::File;
use std::io::Read;
use std::path::Path;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::ZipArchive;

use crate::shield::signatures::ThreatSignatures;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicatorDto {
    pub category: String,
    pub title: String,
    pub severity: String,
    pub weight: u32,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSecurityReportDto {
    pub filepath: String,
    pub filename: String,
    pub sha256: String,
    pub threat_score: u32,
    pub threat_level: String,
    pub is_clean: bool,
    pub entropy: f64,
    pub indicators: Vec<ThreatIndicatorDto>,
    pub file_size: u64,
}

pub struct StaticBytecodeAnalyzer {
    signatures: ThreatSignatures,
}

impl StaticBytecodeAnalyzer {
    pub fn new() -> Self {
        Self {
            signatures: ThreatSignatures::new(),
        }
    }

    pub fn compute_sha256(&self, path: &Path) -> String {
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return String::new(),
        };
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 65536];
        while let Ok(count) = file.read(&mut buffer) {
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        let result = hasher.finalize();
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }

    pub fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mut entropy = 0.0;
        let len = data.len() as f64;
        let mut frequencies = [0usize; 256];

        for &b in data {
            frequencies[b as usize] += 1;
        }

        for &f in &frequencies {
            if f > 0 {
                let p = (f as f64) / len;
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    fn scan_binary_slice(&self, data: &[u8], subfile: &str, indicators: &mut Vec<ThreatIndicatorDto>, score: &mut u32) {
        if self.signatures.ignored_prefixes.iter().any(|p| subfile.starts_with(p)) {
            return;
        }

        for _ in self.signatures.webhook_pattern.find_iter(data) {
            indicators.push(ThreatIndicatorDto {
                category: "EXFILTRATION".to_string(),
                title: "Discord Webhook Exfiltration Endpoint".to_string(),
                severity: "CRITICAL".to_string(),
                weight: 40,
                location: subfile.to_string(),
            });
            *score += 40;
        }

        for _ in self.signatures.telegram_bot_pattern.find_iter(data) {
            indicators.push(ThreatIndicatorDto {
                category: "EXFILTRATION".to_string(),
                title: "Telegram Bot C2 Exfiltration Gateway".to_string(),
                severity: "CRITICAL".to_string(),
                weight: 40,
                location: subfile.to_string(),
            });
            *score += 40;
        }

        for _ in self.signatures.pastebin_pattern.find_iter(data) {
            indicators.push(ThreatIndicatorDto {
                category: "DROPPER".to_string(),
                title: "Remote Payload Paste Gateway".to_string(),
                severity: "HIGH".to_string(),
                weight: 25,
                location: subfile.to_string(),
            });
            *score += 25;
        }

        for &target in &self.signatures.stealer_targets {
            if data.windows(target.len()).any(|w| w == target) {
                let label = String::from_utf8_lossy(target).to_string();
                indicators.push(ThreatIndicatorDto {
                    category: "STEALER".to_string(),
                    title: format!("Access Routine Targeting Vault: {}", label),
                    severity: "HIGH".to_string(),
                    weight: 35,
                    location: subfile.to_string(),
                });
                *score += 35;
            }
        }

        for &rce in &self.signatures.rce_commands {
            if data.windows(rce.len()).any(|w| w == rce) {
                let label = String::from_utf8_lossy(rce).to_string();
                indicators.push(ThreatIndicatorDto {
                    category: "RCE".to_string(),
                    title: format!("Direct Shell Execution Vector: {}", label),
                    severity: "CRITICAL".to_string(),
                    weight: 45,
                    location: subfile.to_string(),
                });
                *score += 45;
            }
        }

        let mut dangerous_api_count = 0;
        for &api in &self.signatures.dangerous_apis {
            if data.windows(api.len()).any(|w| w == api) {
                dangerous_api_count += 1;
            }
        }

        if dangerous_api_count >= 3 {
            indicators.push(ThreatIndicatorDto {
                category: "SUSPICIOUS_API".to_string(),
                title: format!("Clustered Invocation of {} Low-Level/Unsafe JVM APIs", dangerous_api_count),
                severity: "MEDIUM".to_string(),
                weight: 20,
                location: subfile.to_string(),
            });
            *score += 20;
        }

        for m in self.signatures.b64_pattern.find_iter(data) {
            if let Ok(decoded) = BASE64.decode(m.as_bytes()) {
                if self.signatures.webhook_pattern.is_match(&decoded) || self.signatures.telegram_bot_pattern.is_match(&decoded) {
                    indicators.push(ThreatIndicatorDto {
                        category: "OBFUSCATION".to_string(),
                        title: "Base64 Encoded Exfiltration C2 Endpoint".to_string(),
                        severity: "CRITICAL".to_string(),
                        weight: 45,
                        location: subfile.to_string(),
                    });
                    *score += 45;
                }
            }
        }
    }

    pub fn analyze_file(&self, path: &Path) -> FileSecurityReportDto {
        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let filepath = path.to_string_lossy().to_string();
        let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
        let sha256 = self.compute_sha256(path);

        if self.signatures.whitelist_hashes.contains(&sha256.as_str()) {
            return FileSecurityReportDto {
                filepath,
                filename,
                sha256,
                threat_score: 0,
                threat_level: "CLEAN".to_string(),
                is_clean: true,
                entropy: 0.0,
                indicators: Vec::new(),
                file_size,
            };
        }

        let mut indicators = Vec::new();
        let mut score: u32 = 0;
        let mut max_entropy: f64 = 0.0;

        if let Ok(file) = File::open(path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                for i in 0..archive.len() {
                    if let Ok(mut zf) = archive.by_index(i) {
                        let name = zf.name().to_string();

                        if name.ends_with(".exe") || name.ends_with(".dll") || name.ends_with(".so") || name.ends_with(".dylib") {
                            indicators.push(ThreatIndicatorDto {
                                category: "NATIVE_DROPPER".to_string(),
                                title: format!("Embedded Native OS Executable: {}", name),
                                severity: "CRITICAL".to_string(),
                                weight: 50,
                                location: name.clone(),
                            });
                            score += 50;
                        }

                        if name.ends_with(".bat") || name.ends_with(".vbs") || name.ends_with(".ps1") || name.ends_with(".cmd") {
                            indicators.push(ThreatIndicatorDto {
                                category: "SCRIPT_DROPPER".to_string(),
                                title: format!("Embedded Native Shell Script: {}", name),
                                severity: "CRITICAL".to_string(),
                                weight: 45,
                                location: name.clone(),
                            });
                            score += 45;
                        }

                        if name.ends_with(".jar") && !name.starts_with("META-INF/") && !name.contains("fabric-") && !name.contains("jars/") && !name.contains("jij/") {
                            indicators.push(ThreatIndicatorDto {
                                category: "NESTED_JAR".to_string(),
                                title: format!("Suspicious Nested Executable Jar Container: {}", name),
                                severity: "HIGH".to_string(),
                                weight: 30,
                                location: name.clone(),
                            });
                            score += 30;
                        }

                        if (name.ends_with(".class") || name.ends_with(".json") || name.ends_with(".txt") || name.ends_with(".xml")) && zf.size() <= 20971520 {
                            let mut buffer = Vec::new();
                            if zf.read_to_end(&mut buffer).is_ok() {
                                if name.ends_with(".class") && buffer.len() > 4096 {
                                    let ent = self.calculate_entropy(&buffer);
                                    if ent > max_entropy {
                                        max_entropy = ent;
                                    }
                                    if ent > 7.96 && !self.signatures.ignored_prefixes.iter().any(|p| name.starts_with(p)) {
                                        indicators.push(ThreatIndicatorDto {
                                            category: "OBFUSCATION".to_string(),
                                            title: format!("High Entropy Encrypted Class Segment (Entropy: {:.2}): {}", ent, name),
                                            severity: "MEDIUM".to_string(),
                                            weight: 25,
                                            location: name.clone(),
                                        });
                                        score += 25;
                                    }
                                }
                                self.scan_binary_slice(&buffer, &name, &mut indicators, &mut score);
                            }
                        }
                    }
                }
            }
        }

        indicators.sort_by(|a, b| b.weight.cmp(&a.weight));
        indicators.dedup_by(|a, b| a.title == b.title && a.location == b.location);

        let clamped_score = score.min(100);
        let threat_level = match clamped_score {
            0 => "CLEAN",
            1..=25 => "LOW",
            26..=50 => "MEDIUM",
            51..=75 => "HIGH",
            _ => "CRITICAL",
        };

        FileSecurityReportDto {
            filepath,
            filename,
            sha256,
            threat_score: clamped_score,
            threat_level: threat_level.to_string(),
            is_clean: clamped_score == 0,
            entropy: (max_entropy * 100.0).round() / 100.0,
            indicators,
            file_size,
        }
    }
}