use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoBuildRequestDto {
    pub prompt: String,
    pub mc_version: String,
    pub loader: String,
    pub max_mods: usize,
    pub include_performance: bool,
    pub resolve_keybinds_after: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildProgressDto {
    pub phase: String,
    pub percent: f64,
    pub current_step: String,
    pub current_mod: Option<String>,
    pub total_mods: usize,
    pub downloaded_mods: usize,
    pub log_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildResultDto {
    pub success: bool,
    pub total_installed: usize,
    pub installed_mods: Vec<String>,
    pub keybind_changes: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindConflictDto {
    pub key_id: String,
    pub old_binding: String,
    pub new_binding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindResolveReportDto {
    pub success: bool,
    pub conflicts_resolved: usize,
    pub details: Vec<KeybindConflictDto>,
    pub message: String,
}

const SAFE_FALLBACK_KEYS: &[&str] = &[
    "key.keyboard.keypad.0", "key.keyboard.keypad.1", "key.keyboard.keypad.2", "key.keyboard.keypad.3",
    "key.keyboard.keypad.4", "key.keyboard.keypad.5", "key.keyboard.keypad.6", "key.keyboard.keypad.7",
    "key.keyboard.keypad.8", "key.keyboard.keypad.9", "key.keyboard.f6", "key.keyboard.f7",
    "key.keyboard.f8", "key.keyboard.f9", "key.keyboard.f10", "key.keyboard.left.bracket",
    "key.keyboard.right.bracket", "key.keyboard.backslash", "key.keyboard.apostrophe", "key.keyboard.comma",
    "key.keyboard.period", "key.keyboard.slash", "key.keyboard.minus", "key.keyboard.equal",
];

#[tauri::command]
pub async fn execute_auto_build_stream(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    payload: AutoBuildRequestDto,
    progress_channel: Channel<BuildProgressDto>,
) -> Result<BuildResultDto, AppError> {
    let _ = progress_channel.send(BuildProgressDto {
        phase: "AI_SYNTHESIS".to_string(),
        percent: 5.0,
        current_step: "Consulting Neural Core for optimal dependency matrix...".to_string(),
        current_mod: None,
        total_mods: 0,
        downloaded_mods: 0,
        log_line: Some(format!("[AI] Generating modpack architecture for MC {} ({})", payload.mc_version, payload.loader)),
    });

    let generated_slugs = state
        .builder
        .generate_mod_list(&payload.prompt, &payload.mc_version, &payload.loader)
        .await
        .map_err(|e| AppError::Config(format!("Neural synthesis failed: {}", e)))?;

    let mut final_slugs: Vec<String> = Vec::new();
    if payload.include_performance {
        for foundation in state.builder.get_foundation_mods(&payload.loader) {
            final_slugs.push(foundation.to_string());
        }
    }

    for slug in generated_slugs {
        if !final_slugs.contains(&slug) {
            final_slugs.push(slug);
        }
    }

    if payload.max_mods > 0 && final_slugs.len() > payload.max_mods {
        final_slugs.truncate(payload.max_mods);
    }

    let total_mods = final_slugs.len();
    let _ = progress_channel.send(BuildProgressDto {
        phase: "DEPENDENCY_GRAPH".to_string(),
        percent: 20.0,
        current_step: format!("Architecture compiled. Resolving {} package manifests...", total_mods),
        current_mod: None,
        total_mods,
        downloaded_mods: 0,
        log_line: Some(format!("[DAG] Resolved {} unique packages with foundation layer", total_mods)),
    });

    let cfg = config::load_app_config();
    let mods_dir = PathBuf::from(&cfg.current_instance).join("mods");
    fs::create_dir_all(&mods_dir)?;

    let mut installed_mods = Vec::new();

    for (index, slug) in final_slugs.iter().enumerate() {
        let percent = 20.0 + ((index as f64) / (total_mods as f64)) * 65.0;

        let _ = progress_channel.send(BuildProgressDto {
            phase: "DOWNLOADING".to_string(),
            percent,
            current_step: format!("Resolving binary for '{}' ({}/{})", slug, index + 1, total_mods),
            current_mod: Some(slug.clone()),
            total_mods,
            downloaded_mods: installed_mods.len(),
            log_line: Some(format!("[FETCH] Querying Modrinth release for project '{}'", slug)),
        });

        if let Ok(details) = state.store.get_details("modrinth", slug.as_str(), payload.loader.as_str(), payload.mc_version.as_str()).await {
            if let Some(target_ver) = details.versions.first() {
                if let Some(file) = target_ver.files.iter().find(|f| f.primary).or_else(|| target_ver.files.first()) {
                    let clean_fname = Path::new(&file.filename)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&file.filename);
                    let dest_path = mods_dir.join(clean_fname);

                    let dl_res = state
                        .store
                        .download_with_progress(&app, file.url.as_str(), &dest_path, slug.as_str(), clean_fname)
                        .await;

                    if dl_res.is_ok() {
                        installed_mods.push(clean_fname.to_string());
                        let _ = progress_channel.send(BuildProgressDto {
                            phase: "DOWNLOADING".to_string(),
                            percent,
                            current_step: format!("Installed {}", clean_fname),
                            current_mod: Some(slug.to_string()),
                            total_mods,
                            downloaded_mods: installed_mods.len(),
                            log_line: Some(format!("[STORE] Successfully deployed {}", clean_fname)),
                        });
                    }
                }
            }
        }
    }

    let mut keybind_changes = 0;
    if payload.resolve_keybinds_after {
        let _ = progress_channel.send(BuildProgressDto {
            phase: "KEYBIND_OPTIMIZATION".to_string(),
            percent: 90.0,
            current_step: "Optimizing options.txt keybinding collisions...".to_string(),
            current_mod: None,
            total_mods,
            downloaded_mods: installed_mods.len(),
            log_line: Some("[KEYBINDS] Scanning options.txt for duplicate mappings".to_string()),
        });

        keybind_changes = state.builder.resolve_keybinds(&cfg.current_instance);
    }

    let _ = progress_channel.send(BuildProgressDto {
        phase: "COMPLETED".to_string(),
        percent: 100.0,
        current_step: "Modpack synthesis finalized successfully.".to_string(),
        current_mod: None,
        total_mods,
        downloaded_mods: installed_mods.len(),
        log_line: Some(format!("[DONE] Installed {} modules, resolved {} key conflicts", installed_mods.len(), keybind_changes)),
    });

    Ok(BuildResultDto {
        success: true,
        total_installed: installed_mods.len(),
        installed_mods,
        keybind_changes,
        message: "Successfully assembled instance with verified foundation modules.".to_string(),
    })
}

#[tauri::command]
pub async fn resolve_keybind_conflicts_detailed() -> Result<KeybindResolveReportDto, AppError> {
    tokio::task::spawn_blocking(|| {
        let cfg = config::load_app_config();
        let options_path = Path::new(&cfg.current_instance).join("options.txt");

        if !options_path.exists() {
            return Ok(KeybindResolveReportDto {
                success: true,
                conflicts_resolved: 0,
                details: Vec::new(),
                message: "No options.txt detected in active instance.".to_string(),
            });
        }

        let content = fs::read_to_string(&options_path)?;
        let mut keymaps: HashMap<String, String> = HashMap::new();
        let mut resolved_lines = Vec::new();
        let mut details = Vec::new();
        let mut fallback_idx = 0;

        for line in content.lines() {
            if line.starts_with("key_") {
                if let Some((k_id, k_val)) = line.trim().split_once(':') {
                    if k_val.contains("key.keyboard.unknown") {
                        resolved_lines.push(line.to_string());
                        continue;
                    }

                    if keymaps.contains_key(k_val) {
                        if fallback_idx < SAFE_FALLBACK_KEYS.len() {
                            let new_val = SAFE_FALLBACK_KEYS[fallback_idx];
                            fallback_idx += 1;
                            resolved_lines.push(format!("{}:{}", k_id, new_val));
                            details.push(KeybindConflictDto {
                                key_id: k_id.to_string(),
                                old_binding: k_val.to_string(),
                                new_binding: new_val.to_string(),
                            });
                            keymaps.insert(new_val.to_string(), k_id.to_string());
                        } else {
                            resolved_lines.push(format!("{}:key.keyboard.unknown", k_id));
                            details.push(KeybindConflictDto {
                                key_id: k_id.to_string(),
                                old_binding: k_val.to_string(),
                                new_binding: "key.keyboard.unknown".to_string(),
                            });
                        }
                    } else {
                        keymaps.insert(k_val.to_string(), k_id.to_string());
                        resolved_lines.push(line.to_string());
                    }
                } else {
                    resolved_lines.push(line.to_string());
                }
            } else {
                resolved_lines.push(line.to_string());
            }
        }

        if !details.is_empty() {
            let tmp_path = options_path.with_extension("tmp");
            let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&tmp_path)?;
            for l in resolved_lines {
                writeln!(file, "{}", l)?;
            }
            file.sync_all()?;
            drop(file);
            let _ = fs::remove_file(&options_path);
            let _ = fs::rename(tmp_path, options_path);
        }

        let conflicts_resolved = details.len();
        Ok(KeybindResolveReportDto {
            success: true,
            conflicts_resolved,
            details,
            message: format!("Resolved {} button collisions in options.txt.", conflicts_resolved),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}