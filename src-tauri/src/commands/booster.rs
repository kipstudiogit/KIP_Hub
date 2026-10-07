use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tauri::State;

use crate::asset_transcoder::{AssetTranscoder, TranscodeProgressDto, TranscodeResultDto};
use crate::chunk_engine::predictive_baker::{ChunkPrebakeProgressDto, ChunkPrebakeResultDto};
use crate::chunk_engine::ChunkAcceleratorStatusDto;
use crate::commands::AppState;
use crate::config;
use crate::error::AppError;
use crate::hardware_booster::{HardwareBoosterConfigDto, HardwareBoosterStatusDto};
use crate::memory_matrix::{MemoryMatrixStatusDto, MemoryPrefaultResultDto};
use crate::network_optimizer::{
    NetworkOptimizer, PingMasterConfigDto, PingMasterMetricDto, VpnDetectionDto,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingMasterStatusDto {
    pub running: bool,
    pub local_port: Option<u16>,
    pub target_host: String,
    pub target_port: u16,
    pub message: String,
    pub vpn_detected: bool,
    pub active_vpn_adapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageCacheWarmupResultDto {
    pub files_warmed: usize,
    pub transferred_mb: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkRingFlushResultDto {
    pub chunks_flushed: usize,
    pub bytes_written: u64,
    pub message: String,
}

#[tauri::command]
pub fn booster_chunk_accelerator_status(
    state: State<'_, AppState>,
) -> Result<ChunkAcceleratorStatusDto, AppError> {
    Ok(state.chunk_engine.get_status())
}

#[tauri::command]
pub fn booster_chunk_accelerator_toggle(
    state: State<'_, AppState>,
    enable: bool,
) -> bool {
    state.chunk_engine.toggle_accelerator(enable)
}

#[tauri::command]
pub async fn booster_chunk_accelerator_prebake_stream(
    state: State<'_, AppState>,
    center_x: i32,
    center_z: i32,
    radius: i32,
    progress_channel: Channel<ChunkPrebakeProgressDto>,
) -> Result<ChunkPrebakeResultDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    let chunk_engine = state.chunk_engine.clone();

    tokio::task::spawn_blocking(move || {
        chunk_engine
            .run_prebake_stream(&instance_path, center_x, center_z, radius, progress_channel)
            .map_err(|e| AppError::Launch(e))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn booster_chunk_accelerator_flush_ring(
    state: State<'_, AppState>,
) -> Result<ChunkRingFlushResultDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    let chunk_engine = state.chunk_engine.clone();

    tokio::task::spawn_blocking(move || {
        let (chunks, bytes) = chunk_engine.flush_ring_buffer(&instance_path);
        Ok(ChunkRingFlushResultDto {
            chunks_flushed: chunks,
            bytes_written: bytes,
            message: format!("Flushed {} pending chunks ({:.2} MB) into active MCA sectors", chunks, (bytes as f64) / (1024.0 * 1024.0)),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn booster_memory_matrix_status(
    state: State<'_, AppState>,
) -> Result<MemoryMatrixStatusDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    Ok(state.memory_matrix.get_status(&instance_path))
}

#[tauri::command]
pub fn booster_memory_matrix_toggle_large_pages(
    state: State<'_, AppState>,
    enable: bool,
) -> bool {
    state.memory_matrix.toggle_large_pages(enable)
}

#[tauri::command]
pub fn booster_memory_matrix_toggle_compact_headers(
    state: State<'_, AppState>,
    enable: bool,
) -> bool {
    state.memory_matrix.toggle_compact_headers(enable)
}

#[tauri::command]
pub async fn booster_memory_matrix_grant_privilege(
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let memory_matrix = state.memory_matrix.clone();
    tokio::task::spawn_blocking(move || {
        memory_matrix.grant_se_lock_privilege().map_err(|e| AppError::Config(e))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn booster_memory_matrix_prefault_cache(
    state: State<'_, AppState>,
) -> Result<MemoryPrefaultResultDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    let memory_matrix = state.memory_matrix.clone();

    tokio::task::spawn_blocking(move || {
        let (files, mb) = memory_matrix.prefault_standby_cache(&instance_path);
        Ok(MemoryPrefaultResultDto {
            files_cached: files,
            transferred_mb: mb,
            message: format!("Mapped {} JAR modules directly into Standby OS Cache ({:.1} MB ready)", files, mb),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn booster_ping_master_get_config(state: State<'_, AppState>) -> PingMasterConfigDto {
    state.net_optimizer.get_config()
}

#[tauri::command]
pub fn booster_ping_master_set_config(
    state: State<'_, AppState>,
    config: PingMasterConfigDto,
) -> bool {
    state.net_optimizer.set_config(config);
    true
}

#[tauri::command]
pub fn booster_ping_master_detect_vpn() -> VpnDetectionDto {
    NetworkOptimizer::detect_vpn_environment()
}

#[tauri::command]
pub async fn booster_ping_master_start(
    state: State<'_, AppState>,
    target_host: String,
    target_port: Option<u16>,
    config: PingMasterConfigDto,
    channel: Channel<PingMasterMetricDto>,
) -> Result<PingMasterStatusDto, AppError> {
    let clean_host = target_host.trim().to_string();
    if clean_host.is_empty() {
        return Err(AppError::Config("Target host cannot be empty.".to_string()));
    }
    let port = target_port.unwrap_or(25565);

    let vpn_info = NetworkOptimizer::detect_vpn_environment();

    let local_port = state
        .net_optimizer
        .start_proxy(clean_host.clone(), port, config, channel)
        .await
        .map_err(|e| AppError::Launch(e))?;

    Ok(PingMasterStatusDto {
        running: true,
        local_port: Some(local_port),
        target_host: clean_host,
        target_port: port,
        message: format!("Zero-Latency PvP Gateway listening on 127.0.0.1:{}", local_port),
        vpn_detected: vpn_info.vpn_active,
        active_vpn_adapter: vpn_info.active_adapter_name,
    })
}

#[tauri::command]
pub fn booster_ping_master_stop(state: State<'_, AppState>) -> Result<bool, AppError> {
    state.net_optimizer.stop();
    Ok(true)
}

#[tauri::command]
pub fn get_hardware_booster_config(state: State<'_, AppState>) -> HardwareBoosterConfigDto {
    state.hardware_booster.get_config()
}

#[tauri::command]
pub fn set_hardware_booster_config(
    state: State<'_, AppState>,
    config: HardwareBoosterConfigDto,
) -> bool {
    state.hardware_booster.set_config(config);
    true
}

#[tauri::command]
pub async fn get_hardware_booster_status(
    state: State<'_, AppState>,
) -> Result<HardwareBoosterStatusDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    let booster = state.hardware_booster.clone();

    tokio::task::spawn_blocking(move || {
        Ok(booster.get_status(&instance_path))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn execute_asset_transcode_stream(
    progress_channel: Channel<TranscodeProgressDto>,
) -> Result<TranscodeResultDto, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);

    tokio::task::spawn_blocking(move || {
        AssetTranscoder::execute_pipeline(&instance_path, progress_channel)
            .map_err(|e| AppError::Launch(e))
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn trigger_manual_defender_bypass(
    state: State<'_, AppState>,
    enable: bool,
) -> Result<bool, AppError> {
    let cfg = config::load_app_config();
    let instance_path = PathBuf::from(&cfg.current_instance);
    let booster = state.hardware_booster.clone();

    tokio::task::spawn_blocking(move || {
        if enable {
            Ok(booster.add_defender_exclusion(&instance_path))
        } else {
            booster.remove_defender_exclusions();
            Ok(true)
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn trigger_manual_working_set_trim(state: State<'_, AppState>) -> bool {
    state.hardware_booster.trim_launcher_working_set()
}

#[tauri::command]
pub async fn trigger_page_cache_warmup(
    state: State<'_, AppState>,
) -> Result<PageCacheWarmupResultDto, AppError> {
    let cfg = config::load_app_config();
    let mods_dir = PathBuf::from(&cfg.current_instance).join("mods");
    let booster = state.hardware_booster.clone();

    tokio::task::spawn_blocking(move || {
        let (files, mb) = booster.warmup_mods_page_cache(&mods_dir);
        Ok(PageCacheWarmupResultDto {
            files_warmed: files,
            transferred_mb: (mb * 10.0).round() / 10.0,
            message: format!("Evaporated I/O lag for {} mods ({:.1} MB read into Standby List)", files, mb),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn trigger_app_cds_dump(
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let cfg = config::load_app_config();
    let app_cds_dir = PathBuf::from(&cfg.current_instance).join("app_cds");
    let jsa_path = app_cds_dir.join("classes.jsa");
    let booster = state.hardware_booster.clone();
    let instance_path = PathBuf::from(&cfg.current_instance);

    tokio::task::spawn_blocking(move || {
        if jsa_path.exists() {
            let _ = std::fs::remove_file(&jsa_path);
        }
        let _ = booster.resolve_app_cds_flags(&instance_path);
        Ok(true)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}