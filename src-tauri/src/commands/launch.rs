use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sysinfo::System;
use tauri::ipc::Channel;
use tauri::State;

use crate::commands::AppState;
use crate::config;
use crate::doctor_manager::parser::JarParser;
use crate::error::AppError;
use crate::system_utils::SystemUtils;
use crate::tool_manager::ToolManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRequestDto {
    pub version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub ram_allocation: Option<i32>,
    pub jvm_args: Option<String>,
    pub resolution: Option<String>,
    pub fullscreen: bool,
    pub bypass_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchResultDto {
    pub success: bool,
    pub message: String,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchProgressDto {
    pub phase: String,
    pub percent: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightIssueDto {
    pub level: String,
    pub title: String,
    pub description: String,
    pub auto_fixable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightReportDto {
    pub ready_to_launch: bool,
    pub java_compatible: bool,
    pub java_version: String,
    pub java_path: String,
    pub issues: Vec<PreflightIssueDto>,
    pub memory_allocated_gb: i32,
    pub total_system_memory_gb: i32,
    pub has_critical_conflicts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoRepairResultDto {
    pub success: bool,
    pub fixed_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionDownloadResultDto {
    pub success: bool,
    pub version: String,
    pub message: String,
}

#[tauri::command]
pub async fn download_minecraft_version(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    version: String,
) -> Result<VersionDownloadResultDto, AppError> {
    let clean_ver = version.trim().to_string();
    if clean_ver.is_empty() {
        return Err(AppError::Config("Target version identifier cannot be empty.".to_string()));
    }

    let cfg = config::load_app_config();
    let mc_dir = PathBuf::from(&cfg.current_instance);
    let instance = Arc::clone(&state.instance);

    instance
        .ensure_version_downloaded(&clean_ver, &mc_dir, &app)
        .await
        .map_err(|e| AppError::Launch(e))?;

    Ok(VersionDownloadResultDto {
        success: true,
        version: clean_ver.clone(),
        message: format!("Minecraft {} successfully acquired and verified in active instance.", clean_ver),
    })
}

#[tauri::command]
pub async fn inspect_preflight(
    state: State<'_, AppState>,
    version: String,
    loader: String,
    check_mods: bool,
) -> Result<PreflightReportDto, AppError> {
    let doctor = Arc::clone(&state.doctor);
    let java = Arc::clone(&state.java);

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = Path::new(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let cfg_dir = mc_dir.join("config");

        let mut sys = System::new_all();
        sys.refresh_memory();
        let total_system_memory_gb = (sys.total_memory() / (1024 * 1024 * 1024)) as i32;

        let (target_major, target_minor, target_patch) =
            crate::java_manager::JavaManager::parse_mc_version(&version);
        let req_major = if target_major >= 25 {
            25
        } else if target_major == 1 && target_minor <= 16 {
            8
        } else if target_major == 1 && (target_minor < 20 || (target_minor == 20 && target_patch < 5)) {
            17
        } else {
            21
        };

        let dummy_ver_data = json!({
            "javaVersion": {
                "majorVersion": req_major
            }
        });

        let java_res = java.resolve_compatible_java(mc_dir, &loader, &version, &dummy_ver_data);
        let (java_compatible, java_path, java_version) = match java_res {
            Ok(p) => (true, p, SystemUtils::get_java_version()),
            Err(_) => (false, String::new(), "Unavailable".to_string()),
        };

        let mut issues = Vec::new();
        let mut has_critical_conflicts = false;

        if !java_compatible {
            issues.push(PreflightIssueDto {
                level: "CRITICAL".to_string(),
                title: "Java Runtime Missing".to_string(),
                description: format!(
                    "OpenJDK {} runtime was not located for Minecraft {} ({}).",
                    req_major, version, loader
                ),
                auto_fixable: true,
            });
        }

        if cfg.ram_allocation > total_system_memory_gb && total_system_memory_gb > 0 {
            issues.push(PreflightIssueDto {
                level: "WARNING".to_string(),
                title: "Excessive Heap Allocation".to_string(),
                description: format!(
                    "Allocated {} GB exceeds physical RAM ({} GB).",
                    cfg.ram_allocation, total_system_memory_gb
                ),
                auto_fixable: true,
            });
        }

        if check_mods && mods_dir.exists() {
            let analysis = doctor.run_analysis(
                mods_dir.to_str().unwrap_or(""),
                cfg_dir.to_str().unwrap_or(""),
                &version,
                &loader,
            );

            if let Some(items) = analysis["issues"].as_array() {
                for item in items {
                    let lvl = item["type"].as_str().unwrap_or("");
                    let text = item["text"].as_str().unwrap_or("");
                    let action = item["action"].as_str().unwrap_or("");
                    if lvl == "CRITICAL" {
                        has_critical_conflicts = true;
                        issues.push(PreflightIssueDto {
                            level: "CRITICAL".to_string(),
                            title: "Mod Incompatibility Collision".to_string(),
                            description: text.to_string(),
                            auto_fixable: action == "DELETE" || action == "DISABLE",
                        });
                    } else if lvl == "WARNING" {
                        issues.push(PreflightIssueDto {
                            level: "WARNING".to_string(),
                            title: "Mod Advisory".to_string(),
                            description: text.to_string(),
                            auto_fixable: action == "DELETE" || action == "DISABLE" || action == "DOWNLOAD",
                        });
                    }
                }
            }
        }

        Ok(PreflightReportDto {
            ready_to_launch: java_compatible && (!has_critical_conflicts || !check_mods),
            java_compatible,
            java_version,
            java_path,
            issues,
            memory_allocated_gb: if cfg.ram_allocation > 0 { cfg.ram_allocation } else { 4 },
            total_system_memory_gb,
            has_critical_conflicts,
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn auto_repair_environment(
    state: State<'_, AppState>,
    version: String,
    loader: String,
) -> Result<AutoRepairResultDto, AppError> {
    let doctor = Arc::clone(&state.doctor);

    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let mc_dir = PathBuf::from(&cfg.current_instance);
        let mods_dir = mc_dir.join("mods");
        let cfg_dir = mc_dir.join("config");
        let saves_dir = mc_dir.join("saves");

        let _ = SystemUtils::kill_zombie_processes();
        let _ = ToolManager::unlock_worlds(saves_dir.to_str().unwrap_or(""));

        let analysis = doctor.run_analysis(
            mods_dir.to_str().unwrap_or(""),
            cfg_dir.to_str().unwrap_or(""),
            &version,
            &loader,
        );

        let mut fixed_count = 0;
        if let Some(items) = analysis["issues"].as_array() {
            for item in items {
                let action = item["action"].as_str().unwrap_or("");
                let target_file = item["target_file"].as_str().unwrap_or("");

                if (action == "DELETE" || action == "DISABLE") && !target_file.is_empty() {
                    let target_path = if let Some(stripped) = target_file.strip_prefix("../") {
                        mc_dir.join(stripped)
                    } else {
                        mods_dir.join(target_file)
                    };

                    if target_path.exists() && target_path.is_file() {
                        let disabled_name = format!("{}.disabled", target_path.to_string_lossy());
                        if fs::rename(&target_path, &disabled_name).is_ok() {
                            fixed_count += 1;
                        }
                    }
                }
            }
        }

        Ok(AutoRepairResultDto {
            success: true,
            fixed_count,
            message: format!("Resolved {} conflict points. System primed for ignition.", fixed_count),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn launch_instance_stream(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    payload: LaunchRequestDto,
    progress_channel: Channel<LaunchProgressDto>,
) -> Result<LaunchResultDto, AppError> {
    let _ = progress_channel.send(LaunchProgressDto {
        phase: "CALIBRATION".to_string(),
        percent: 10.0,
        message: "Auditing launch configurations and instance integrity...".to_string(),
    });

    let mut cfg = config::load_app_config();

    if let Some(ram) = payload.ram_allocation {
        if ram >= 0 {
            cfg.ram_allocation = ram;
        }
    }
    if let Some(ref res) = payload.resolution {
        if !res.trim().is_empty() {
            cfg.game_resolution = res.clone();
        }
    }
    if let Some(ref custom_args) = payload.jvm_args {
        cfg.custom_jvm_args = custom_args.clone();
    }
    cfg.game_fullscreen = payload.fullscreen;
    config::save_app_config(&cfg);

    let mc_dir = cfg.current_instance.clone();
    let mc_dir_buf = PathBuf::from(&mc_dir);
    let saves_path = Path::new(&mc_dir).join("saves");
    let backups_path = Path::new(&mc_dir).join("backups_devkit");
    let auto_backup = cfg.auto_backup;

    tokio::task::spawn_blocking(move || {
        ToolManager::unlock_worlds(saves_path.to_str().unwrap_or(""));
        SystemUtils::kill_zombie_processes();
        if auto_backup {
            ToolManager::create_backup(
                saves_path.to_str().unwrap_or(""),
                backups_path.to_str().unwrap_or(""),
            );
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?;

    let booster_cfg = state.hardware_booster.get_config();

    if booster_cfg.timer_resolution_enabled {
        state.hardware_booster.enable_high_resolution_timer();
    }

    if booster_cfg.defender_bypass_enabled {
        state.hardware_booster.add_defender_exclusion(&mc_dir_buf);
    }

    if booster_cfg.page_cache_warmup_enabled {
        let mods_path = mc_dir_buf.join("mods");
        state.hardware_booster.warmup_mods_page_cache(&mods_path);
    }

    state.memory_matrix.prefault_standby_cache(&mc_dir_buf);
    let kmm_flags = state.memory_matrix.resolve_jvm_memory_flags(&mc_dir_buf);

    let mut injected_jvm_args = String::new();
    for flag in kmm_flags {
        injected_jvm_args.push_str(&format!(" {} ", flag));
    }

    if booster_cfg.app_cds_enabled {
        let app_cds_flags = state.hardware_booster.resolve_app_cds_flags(&mc_dir_buf);
        for flag in app_cds_flags {
            injected_jvm_args.push_str(&format!(" {} ", flag));
        }
    }

    if booster_cfg.crac_acceleration_enabled {
        let crac_flags = state.hardware_booster.resolve_crac_flags(&mc_dir_buf);
        for flag in crac_flags {
            injected_jvm_args.push_str(&format!(" {} ", flag));
        }
    }

    let target_version = payload.version.clone();
    let target_loader = payload.loader.clone();
    let mods_dir = mc_dir_buf.join("mods");

    if mods_dir.exists() && !payload.bypass_checks {
        let (major, minor, patch) = crate::java_manager::JavaManager::parse_mc_version(&target_version);
        let max_allowed_class = if major >= 25 {
            69
        } else if major == 1 && minor <= 16 {
            52
        } else if major == 1 && (minor < 20 || (minor == 20 && patch < 5)) {
            61
        } else {
            65
        };

        let parser = JarParser::new();
        if let Ok(entries) = fs::read_dir(&mods_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.is_file() {
                    let fname = entry.file_name().to_string_lossy().to_string();
                    if fname.ends_with(".jar") && !fname.ends_with(".disabled") {
                        let desc = parser.parse(&p);
                        let mut should_disable = false;

                        if desc.bytecode.baseline_class_version > max_allowed_class {
                            should_disable = true;
                        }

                        if (target_loader == "forge" || target_loader == "neoforge")
                            && (desc.loaders.contains("fabric") || desc.loaders.contains("quilt"))
                            && !desc.loaders.contains("forge")
                            && !desc.loaders.contains("neoforge")
                        {
                            should_disable = true;
                        }

                        if target_loader == "fabric"
                            && desc.loaders.contains("forge")
                            && !desc.loaders.contains("fabric")
                            && !desc.loaders.contains("quilt")
                        {
                            should_disable = true;
                        }

                        if should_disable {
                            let disabled_target = format!("{}.disabled", p.to_string_lossy());
                            let _ = fs::rename(&p, disabled_target);
                        }
                    }
                }
            }
        }
    }

    let _ = progress_channel.send(LaunchProgressDto {
        phase: "AUTHENTICATION".to_string(),
        percent: 30.0,
        message: "Verifying user license credentials...".to_string(),
    });

    let token = config::get_secret("ms_access_token");
    let mut account = json!({
        "name": "Player",
        "uuid": "00000000-0000-0000-0000-000000000000",
        "access_token": "0"
    });

    if !token.is_empty() {
        if let Ok(acc) = state.auth.authenticate_minecraft(&token).await {
            account = acc;
        }
    }

    if account["access_token"].as_str() == Some("0") {
        let username = if !cfg.offline_username.trim().is_empty() {
            cfg.offline_username.clone()
        } else {
            let gen = format!("Operator_{}", &uuid::Uuid::new_v4().simple().to_string()[..4]);
            cfg.offline_username = gen.clone();
            config::save_app_config(&cfg);
            gen
        };
        account["name"] = json!(username);
    }

    let _ = progress_channel.send(LaunchProgressDto {
        phase: "RESOLVING_LIBRARIES".to_string(),
        percent: 60.0,
        message: "Synchronizing classpaths, loaders and runtime binaries...".to_string(),
    });

    let loader_ver = payload.loader_version.unwrap_or_default();
    let instance = Arc::clone(&state.instance);

    let clean_extra = injected_jvm_args.trim();
    let extra_arg_ref = if clean_extra.is_empty() {
        None
    } else {
        Some(clean_extra)
    };

    let launch_result = instance
        .launch_game(
            &target_version,
            &target_loader,
            &loader_ver,
            &mc_dir,
            &account,
            extra_arg_ref,
            &app,
        )
        .await
        .map_err(|err_msg| AppError::Launch(err_msg))?;

    let pid = launch_result.1;
    state.monitor.set_game_pid(pid);

    if booster_cfg.process_priority_boost {
        state.hardware_booster.boost_game_process_priority(pid);
    }

    if booster_cfg.trim_launcher_memory {
        state.hardware_booster.trim_launcher_working_set();
    }

    let _ = progress_channel.send(LaunchProgressDto {
        phase: "IGNITION".to_string(),
        percent: 100.0,
        message: "Minecraft bootstrap launched successfully.".to_string(),
    });

    Ok(LaunchResultDto {
        success: true,
        message: launch_result.0,
        pid: Some(pid),
    })
}