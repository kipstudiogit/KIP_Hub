pub mod parser;
pub mod remediation;
pub mod rules;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_manager::ApiManager;
use parser::{JarParser, ModDescriptor};
use remediation::RemediationEngine;
use rules::RulesEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeStatsDto {
    pub java_8: usize,
    pub java_17: usize,
    pub java_21: usize,
    pub java_25: usize,
    pub max_detected_major: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorIssueDto {
    pub id: String,
    pub r#type: String,
    pub category: String,
    pub title: String,
    pub text: String,
    pub action: String,
    pub target_file: String,
    pub target_slug: String,
    pub extra_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorAnalysisReportDto {
    pub is_clean: bool,
    pub health_score: i32,
    pub risk_level: String,
    pub total_checked: usize,
    pub target_mc: String,
    pub target_loader: String,
    pub bytecode_stats: BytecodeStatsDto,
    pub issues: Vec<DoctorIssueDto>,
}

pub struct DoctorManager {
    parser: JarParser,
    rules: RulesEngine,
    re_cfg_ext: Regex,
}

impl DoctorManager {
    pub fn new() -> Self {
        Self {
            parser: JarParser::new(),
            rules: RulesEngine::new(),
            re_cfg_ext: Regex::new(r"\.(json|toml|json5|cfg|txt)$").unwrap(),
        }
    }

    fn calculate_expected_baseline(major: u32, minor: u32, patch: u32) -> u16 {
        if major >= 25 {
            69
        } else if major == 1 && minor <= 16 {
            52
        } else if major == 1 && (minor < 20 || (minor == 20 && patch < 5)) {
            61
        } else {
            65
        }
    }

    pub fn run_analysis(&self, mods_dir: &str, config_dir: &str, mc_version: &str, loader: &str) -> Value {
        let m_dir = Path::new(mods_dir);
        let target_mc_ver = if !mc_version.trim().is_empty() { mc_version.trim().to_string() } else { "1.21.4".to_string() };
        let target_loader_name = if !loader.trim().is_empty() { loader.trim().to_lowercase() } else { "vanilla".to_string() };

        if !m_dir.exists() {
            let empty_report = DoctorAnalysisReportDto {
                is_clean: false,
                health_score: 0,
                risk_level: "CRITICAL".to_string(),
                total_checked: 0,
                target_mc: target_mc_ver,
                target_loader: target_loader_name,
                bytecode_stats: BytecodeStatsDto { java_8: 0, java_17: 0, java_21: 0, java_25: 0, max_detected_major: 0 },
                issues: vec![DoctorIssueDto {
                    id: "no_folder".to_string(),
                    r#type: "CRITICAL".to_string(),
                    category: "ENVIRONMENT".to_string(),
                    title: "Missing Mods Container".to_string(),
                    text: "The 'mods' folder was not found in the active Minecraft instance.".to_string(),
                    action: "NONE".to_string(),
                    target_file: String::new(),
                    target_slug: String::new(),
                    extra_info: "Initialize an instance directory before running Doctor.".to_string(),
                }],
            };
            return serde_json::to_value(empty_report).unwrap_or_default();
        }

        let (target_major, target_minor, target_patch) = crate::java_manager::JavaManager::parse_mc_version(&target_mc_ver);
        let expected_baseline_class = Self::calculate_expected_baseline(target_major, target_minor, target_patch);

        let mut issues: Vec<DoctorIssueDto> = Vec::new();
        let mut installed_mods: HashMap<String, ModDescriptor> = HashMap::new();
        let mut virtual_tokens: HashSet<String> = HashSet::new();

        virtual_tokens.insert("minecraft".to_string());
        virtual_tokens.insert("java".to_string());
        virtual_tokens.insert("fabricloader".to_string());
        virtual_tokens.insert("quilt_loader".to_string());
        virtual_tokens.insert("forge".to_string());
        virtual_tokens.insert("neoforge".to_string());

        let mut java_8_count = 0;
        let mut java_17_count = 0;
        let mut java_21_count = 0;
        let mut java_25_count = 0;
        let mut max_detected_major = 52;
        let mut total_checked = 0;

        let entries = match fs::read_dir(m_dir) {
            Ok(e) => e,
            Err(e) => {
                let err_report = DoctorAnalysisReportDto {
                    is_clean: false,
                    health_score: 0,
                    risk_level: "CRITICAL".to_string(),
                    total_checked: 0,
                    target_mc: target_mc_ver,
                    target_loader: target_loader_name,
                    bytecode_stats: BytecodeStatsDto { java_8: 0, java_17: 0, java_21: 0, java_25: 0, max_detected_major: 0 },
                    issues: vec![DoctorIssueDto {
                        id: "fs_err".to_string(),
                        r#type: "CRITICAL".to_string(),
                        category: "ENVIRONMENT".to_string(),
                        title: "IO Error".to_string(),
                        text: e.to_string(),
                        action: "NONE".to_string(),
                        target_file: String::new(),
                        target_slug: String::new(),
                        extra_info: String::new(),
                    }],
                };
                return serde_json::to_value(err_report).unwrap_or_default();
            }
        };

        for entry in entries.filter_map(|e| e.ok()) {
            let fp = entry.path();
            if !fp.is_file() { continue; }
            let f = entry.file_name().to_string_lossy().to_string();
            if f.ends_with(".disabled") { continue; }

            if !f.ends_with(".jar") {
                let fl = f.to_lowercase();
                if fl.ends_with(".zip") || fl.ends_with(".rar") || fl.ends_with(".txt") || fl.ends_with(".json") {
                    issues.push(DoctorIssueDto {
                        id: format!("inv_{}", f),
                        r#type: "OPTIMIZATION".to_string(),
                        category: "CLEANUP".to_string(),
                        title: "Non-Executable Package in Mods".to_string(),
                        text: format!("Residual archive or payload '{}' detected in mods directory.", f),
                        action: "DISABLE".to_string(),
                        target_file: f,
                        target_slug: String::new(),
                        extra_info: "Only compiled Java archive containers belong in the mods folder.".to_string(),
                    });
                }
                continue;
            }

            total_checked += 1;
            let m = self.parser.parse(&fp);

            if m.is_corrupted {
                issues.push(DoctorIssueDto {
                    id: format!("corr_{}", f),
                    r#type: "CRITICAL".to_string(),
                    category: "CORRUPTED".to_string(),
                    title: "Corrupted Java Archive".to_string(),
                    text: format!("File '{}' has invalid ZIP headers or truncated class streams.", f),
                    action: "DISABLE".to_string(),
                    target_file: f,
                    target_slug: String::new(),
                    extra_info: "Corrupted archives cause immediate java.util.zip.ZipException crashes.".to_string(),
                });
                continue;
            }

            if m.bytecode.baseline_class_version > max_detected_major {
                max_detected_major = m.bytecode.baseline_class_version;
            }

            match m.bytecode.baseline_class_version {
                0..=52 => java_8_count += 1,
                53..=61 => java_17_count += 1,
                62..=65 => java_21_count += 1,
                _ => java_25_count += 1,
            }

            if m.id != "?" {
                for p in &m.provides {
                    virtual_tokens.insert(p.clone());
                }

                if m.id == "fabric-api" || m.provides.contains("fabric-api") {
                    for sub in &self.rules.fapi_submodules {
                        virtual_tokens.insert(sub.to_string());
                    }
                }
                if m.id == "qsl" || m.provides.contains("qsl") || m.id.contains("quilted_fabric_api") {
                    for sub in &self.rules.qsl_submodules {
                        virtual_tokens.insert(sub.to_string());
                    }
                    for sub in &self.rules.fapi_submodules {
                        virtual_tokens.insert(sub.to_string());
                    }
                }
                if m.id == "fabric-language-kotlin" {
                    virtual_tokens.insert("kotlin".to_string());
                    virtual_tokens.insert("org_jetbrains_kotlin_kotlin-stdlib".to_string());
                }
                if m.id == "architectury" {
                    virtual_tokens.insert("architectury-api".to_string());
                }
                if m.id == "cloth-config" || m.id == "cloth_config" {
                    virtual_tokens.insert("cloth-config2".to_string());
                }

                if let Some(existing) = installed_mods.get(&m.id) {
                    if existing.version != "1.0.0" && m.version != "1.0.0" && existing.version != m.version {
                        let v_existing = self.rules.parse_semver(&existing.version);
                        let v_new = self.rules.parse_semver(&m.version);

                        if v_new > v_existing {
                            issues.push(DoctorIssueDto {
                                id: format!("dup_{}_{}", m.id, existing.filename),
                                r#type: "WARNING".to_string(),
                                category: "DUPLICATE".to_string(),
                                title: "Duplicate Mod Revision Detected".to_string(),
                                text: format!("Found older '{}' ({}) alongside newer '{}' ({}).", existing.filename, existing.version, m.filename, m.version),
                                action: "DISABLE".to_string(),
                                target_file: existing.filename.clone(),
                                target_slug: String::new(),
                                extra_info: "Retaining highest SemVer build to prevent duplicate class definition crashes.".to_string(),
                            });
                            installed_mods.insert(m.id.clone(), m);
                        } else {
                            issues.push(DoctorIssueDto {
                                id: format!("dup_{}_{}", m.id, m.filename),
                                r#type: "WARNING".to_string(),
                                category: "DUPLICATE".to_string(),
                                title: "Duplicate Mod Revision Detected".to_string(),
                                text: format!("Redundant older binary '{}' ({}) detected alongside '{}' ({}).", m.filename, m.version, existing.filename, existing.version),
                                action: "DISABLE".to_string(),
                                target_file: m.filename.clone(),
                                target_slug: String::new(),
                                extra_info: "Disabling duplicate legacy binary.".to_string(),
                            });
                        }
                    }
                } else {
                    installed_mods.insert(m.id.clone(), m);
                }
            } else if !m.is_coremod {
                issues.push(DoctorIssueDto {
                    id: format!("unk_{}", f),
                    r#type: "OPTIMIZATION".to_string(),
                    category: "LOADER".to_string(),
                    title: "Unidentified Java Package".to_string(),
                    text: format!("'{}' does not declare standard Fabric/Quilt/Forge metadata descriptors.", f),
                    action: "DISABLE".to_string(),
                    target_file: f,
                    target_slug: String::new(),
                    extra_info: "May be a server-only plugin, library or legacy Java agent.".to_string(),
                });
            }
        }

        for m in installed_mods.values() {
            if m.bytecode.baseline_class_version > expected_baseline_class {
                let required_jvm = match m.bytecode.baseline_class_version {
                    65 => "Java 21",
                    66..=69 => "Java 25",
                    61 => "Java 17",
                    _ => "Newer JVM",
                };
                let resolved_slug = self.rules.resolve_slug(&m.id);
                issues.push(DoctorIssueDto {
                    id: format!("bytecode_{}", m.filename),
                    r#type: "CRITICAL".to_string(),
                    category: "BYTECODE".to_string(),
                    title: "Fatal JVM Bytecode Mismatch".to_string(),
                    text: format!("'{}' requires {} (class {}), but target runtime is class {}.", m.filename, required_jvm, m.bytecode.baseline_class_version, expected_baseline_class),
                    action: "DISABLE".to_string(),
                    target_file: m.filename.clone(),
                    target_slug: resolved_slug,
                    extra_info: "JVM will abort with java.lang.UnsupportedClassVersionError upon launch.".to_string(),
                });
            }

            if target_loader_name == "fabric" {
                if m.loaders.contains("forge") && !m.loaders.contains("fabric") && !m.loaders.contains("quilt") {
                    issues.push(DoctorIssueDto {
                        id: format!("wrong_loader_{}", m.filename),
                        r#type: "CRITICAL".to_string(),
                        category: "LOADER".to_string(),
                        title: "Cross-Loader Incompatibility".to_string(),
                        text: format!("'{}' targets Forge and cannot execute on Fabric bootstrap.", m.filename),
                        action: "DISABLE".to_string(),
                        target_file: m.filename.clone(),
                        target_slug: String::new(),
                        extra_info: "Forge mods rely on FML interfaces absent in Fabric.".to_string(),
                    });
                }
            } else if target_loader_name == "quilt" {
                if m.loaders.contains("forge") && !m.loaders.contains("fabric") && !m.loaders.contains("quilt") {
                    issues.push(DoctorIssueDto {
                        id: format!("wrong_loader_{}", m.filename),
                        r#type: "CRITICAL".to_string(),
                        category: "LOADER".to_string(),
                        title: "Cross-Loader Incompatibility".to_string(),
                        text: format!("'{}' targets Forge and cannot execute on Quilt.", m.filename),
                        action: "DISABLE".to_string(),
                        target_file: m.filename.clone(),
                        target_slug: String::new(),
                        extra_info: "Quilt supports Fabric natively, but is incompatible with Forge.".to_string(),
                    });
                }
            } else if (target_loader_name == "forge" || target_loader_name == "neoforge") && (m.loaders.contains("fabric") || m.loaders.contains("quilt")) && !m.loaders.contains("forge") && !m.loaders.contains("neoforge") {
                issues.push(DoctorIssueDto {
                    id: format!("wrong_loader_{}", m.filename),
                    r#type: "CRITICAL".to_string(),
                    category: "LOADER".to_string(),
                    title: "Cross-Loader Incompatibility".to_string(),
                    text: format!("'{}' targets Fabric/Quilt and cannot execute on Forge/NeoForge.", m.filename),
                    action: "DISABLE".to_string(),
                    target_file: m.filename.clone(),
                    target_slug: String::new(),
                    extra_info: "Fabric mods require Knot loader environment.".to_string(),
                });
            }

            if let Some(req_mc) = m.depends.get("minecraft") {
                if !self.rules.matches_minecraft_version(&target_mc_ver, req_mc) {
                    let resolved_slug = self.rules.resolve_slug(&m.id);
                    issues.push(DoctorIssueDto {
                        id: format!("mc_ver_{}", m.filename),
                        r#type: "ERROR".to_string(),
                        category: "LOADER".to_string(),
                        title: "Minecraft Version Mismatch".to_string(),
                        text: format!("'{}' specifies Minecraft dependency '{}', active target is {}.", m.filename, req_mc, target_mc_ver),
                        action: "DISABLE".to_string(),
                        target_file: m.filename.clone(),
                        target_slug: resolved_slug,
                        extra_info: "Method obfuscation maps change between Minecraft releases.".to_string(),
                    });
                }
            }

            for (dep_id, req_range) in &m.depends {
                if self.rules.is_internal_module(dep_id) {
                    continue;
                }

                if !virtual_tokens.contains(dep_id) && !installed_mods.contains_key(dep_id) {
                    let mapped_slug = self.rules.resolve_slug(dep_id);
                    issues.push(DoctorIssueDto {
                        id: format!("dep_missing_{}_{}", m.id, dep_id),
                        r#type: "ERROR".to_string(),
                        category: "DEPENDENCY".to_string(),
                        title: "Missing Required Companion Library".to_string(),
                        text: format!("Module '{}' strictly requires companion '{}' ({}).", m.name, mapped_slug, req_range),
                        action: "DOWNLOAD".to_string(),
                        target_file: String::new(),
                        target_slug: mapped_slug,
                        extra_info: format!("Declared by: {} -> {}", m.id, dep_id),
                    });
                } else if let Some(installed_dep) = installed_mods.get(dep_id) {
                    if req_range != "*" && !self.rules.matches_semver_range(&installed_dep.version, req_range) {
                        let mapped_slug = self.rules.resolve_slug(dep_id);
                        issues.push(DoctorIssueDto {
                            id: format!("dep_outdated_{}_{}", m.id, dep_id),
                            r#type: "WARNING".to_string(),
                            category: "DEPENDENCY".to_string(),
                            title: "Outdated Dependency Version".to_string(),
                            text: format!("'{}' requires '{}' {}, but installed version is {}.", m.name, installed_dep.name, req_range, installed_dep.version),
                            action: "UPGRADE".to_string(),
                            target_file: installed_dep.filename.clone(),
                            target_slug: mapped_slug,
                            extra_info: "Outdated APIs can trigger NoSuchMethodError during runtime.".to_string(),
                        });
                    }
                }
            }

            for (break_id, _) in &m.breaks {
                if let Some(conflicting_mod) = installed_mods.get(break_id) {
                    issues.push(DoctorIssueDto {
                        id: format!("conflict_{}_{}", m.id, break_id),
                        r#type: "CRITICAL".to_string(),
                        category: "COLLISION".to_string(),
                        title: "Declared Incompatibility Collision".to_string(),
                        text: format!("'{}' declares an unresolvable collision with '{}'.", m.name, conflicting_mod.name),
                        action: "DISABLE".to_string(),
                        target_file: conflicting_mod.filename.clone(),
                        target_slug: String::new(),
                        extra_info: format!("Descriptor in {} explicitly breaks {}.", m.id, break_id),
                    });
                }
            }
        }

        let optifine_installed = installed_mods.values().any(|m| m.filename.to_lowercase().contains("optifine"));
        let sodium_installed = installed_mods.contains_key("sodium") || installed_mods.contains_key("embeddium") || installed_mods.contains_key("rubidium");
        let optifabric_installed = installed_mods.contains_key("optifabric");
        let indium_installed = installed_mods.contains_key("indium");
        let qfapi_installed = installed_mods.contains_key("quilted_fabric_api") || installed_mods.values().any(|m| m.filename.to_lowercase().contains("qfapi"));
        let fapi_installed = installed_mods.contains_key("fabric-api") || installed_mods.values().any(|m| m.filename.to_lowercase().starts_with("fabric-api"));

        if optifine_installed && sodium_installed {
            let optifine_target = installed_mods.values().find(|m| m.filename.to_lowercase().contains("optifine")).map(|m| m.filename.clone()).unwrap_or_default();
            issues.push(DoctorIssueDto {
                id: "collision_optifine_sodium".to_string(),
                r#type: "CRITICAL".to_string(),
                category: "COLLISION".to_string(),
                title: "Fatal Rendering Pipeline Collision".to_string(),
                text: "OptiFine and Sodium/Embeddium rewrite identical chunk renderers. The game will crash on startup.".to_string(),
                action: "DISABLE".to_string(),
                target_file: optifine_target,
                target_slug: String::new(),
                extra_info: "Sodium replaces chunk baking pipelines; OptiFine ASM hooks will fail.".to_string(),
            });
        }

        if optifine_installed && target_loader_name == "fabric" && !optifabric_installed {
            issues.push(DoctorIssueDto {
                id: "optifabric_missing".to_string(),
                r#type: "CRITICAL".to_string(),
                category: "DEPENDENCY".to_string(),
                title: "Missing OptiFabric Bridge Layer".to_string(),
                text: "OptiFine on Fabric requires OptiFabric transformation bridge.".to_string(),
                action: "DOWNLOAD".to_string(),
                target_file: String::new(),
                target_slug: "optifabric".to_string(),
                extra_info: "OptiFine cannot hook into Knot classloader without transformation shims.".to_string(),
            });
        }

        if sodium_installed && !indium_installed {
            let requires_frapi = installed_mods.contains_key("continuity")
                || installed_mods.contains_key("chipped")
                || installed_mods.contains_key("athena")
                || installed_mods.contains_key("lambdabettergrass")
                || installed_mods.contains_key("campanion");

            if requires_frapi {
                issues.push(DoctorIssueDto {
                    id: "indium_missing_frapi".to_string(),
                    r#type: "CRITICAL".to_string(),
                    category: "DEPENDENCY".to_string(),
                    title: "Missing FRAPI Shader Shim (Indium)".to_string(),
                    text: "Sodium disables standard Fabric Rendering API. Connected textures require Indium.".to_string(),
                    action: "DOWNLOAD".to_string(),
                    target_file: String::new(),
                    target_slug: "indium".to_string(),
                    extra_info: "Models with custom vertex formats will crash during block tessellation.".to_string(),
                });
            }
        }

        if qfapi_installed && fapi_installed {
            let fapi_target = installed_mods.values().find(|m| m.filename.to_lowercase().starts_with("fabric-api")).map(|m| m.filename.clone()).unwrap_or_default();
            issues.push(DoctorIssueDto {
                id: "collision_qfapi_fapi".to_string(),
                r#type: "CRITICAL".to_string(),
                category: "COLLISION".to_string(),
                title: "Dual Fabric API Implementation Collision".to_string(),
                text: "Quilted Fabric API and standard Fabric API are installed together. QFAPI already contains full FAPI.".to_string(),
                action: "DISABLE".to_string(),
                target_file: fapi_target,
                target_slug: String::new(),
                extra_info: "Causes duplicate mixin registration and crash during early mod init.".to_string(),
            });
        }

        let cfg_dir = Path::new(config_dir);
        if cfg_dir.exists() {
            if let Ok(c_entries) = fs::read_dir(cfg_dir) {
                let ignored_cfgs = ["forge", "fabric", "quilt", "neoforge", "minecraft", "options", "splash", "fml"];
                for c_entry in c_entries.filter_map(|e| e.ok()) {
                    let cf_name = c_entry.file_name().to_string_lossy().to_string();
                    if cf_name.ends_with(".json") || cf_name.ends_with(".toml") || cf_name.ends_with(".json5") || cf_name.ends_with(".cfg") {
                        let base = self.re_cfg_ext.replace(&cf_name, "").to_string();
                        let cfg_id = base.split('-').next().unwrap_or("").to_lowercase();
                        if !virtual_tokens.contains(&cfg_id) && !installed_mods.contains_key(&cfg_id) && !ignored_cfgs.contains(&cfg_id.as_str()) {
                            issues.push(DoctorIssueDto {
                                id: format!("cfg_orphan_{}", cf_name),
                                r#type: "OPTIMIZATION".to_string(),
                                category: "CLEANUP".to_string(),
                                title: "Orphaned Configuration File".to_string(),
                                text: format!("Residual configuration '{}' from an uninstalled mod.", cf_name),
                                action: "DISABLE".to_string(),
                                target_file: format!("../config/{}", cf_name),
                                target_slug: String::new(),
                                extra_info: "Unused configuration residue adds disk overhead.".to_string(),
                            });
                        }
                    }
                }
            }
        }

        let mut penalty = 0;
        for issue in &issues {
            match issue.r#type.as_str() {
                "CRITICAL" => penalty += 35,
                "ERROR" => penalty += 18,
                "WARNING" => penalty += 6,
                "OPTIMIZATION" => penalty += 2,
                _ => penalty += 1,
            }
        }

        let health_score = 100i32.saturating_sub(penalty).max(0);
        let risk_level = match health_score {
            95..=100 => "OPTIMAL",
            75..=94 => "STABLE",
            40..=74 => "ELEVATED_RISK",
            _ => "CRITICAL",
        };

        let report = DoctorAnalysisReportDto {
            is_clean: issues.is_empty(),
            health_score,
            risk_level: risk_level.to_string(),
            total_checked,
            target_mc: target_mc_ver,
            target_loader: target_loader_name,
            bytecode_stats: BytecodeStatsDto {
                java_8: java_8_count,
                java_17: java_17_count,
                java_21: java_21_count,
                java_25: java_25_count,
                max_detected_major,
            },
            issues,
        };

        serde_json::to_value(report).unwrap_or_default()
    }

    pub async fn apply_fixes(&self, mods_dir: &str, api_manager: &ApiManager, issues: &[Value], mc_version: &str, loader: &str) -> Value {
        let active_mc = if !mc_version.is_empty() { mc_version } else { "1.21.4" };
        let active_loader = if !loader.is_empty() { loader } else { "vanilla" };

        RemediationEngine::apply(mods_dir, api_manager, issues, active_mc, active_loader).await
    }
}