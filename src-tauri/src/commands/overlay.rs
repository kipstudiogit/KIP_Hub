use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State, Window};

use crate::commands::AppState;
use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaypointRecordDto {
    pub id: String,
    pub name: String,
    pub dimension: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub note: String,
}

fn get_overlay_storage_dir() -> PathBuf {
    let dir = config::get_app_data_dir().join("overlay");
    let _ = fs::create_dir_all(&dir);
    dir
}

#[tauri::command]
pub fn overlay_set_active(
    window: Window,
    _state: State<'_, AppState>,
    active: bool,
    has_pinned: bool,
) -> Result<bool, AppError> {
    if active {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_fullscreen(true);
        let _ = window.set_always_on_top(true);
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_focus();
    } else if has_pinned {
        let _ = window.set_fullscreen(true);
        let _ = window.set_always_on_top(true);
        let _ = window.set_ignore_cursor_events(true);
    } else {
        let _ = window.set_fullscreen(false);
        let _ = window.set_resizable(true);
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 1320.0,
            height: 880.0,
        }));
        let _ = window.center();
        let _ = window.set_always_on_top(false);
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_focus();
    }
    Ok(active)
}

#[tauri::command]
pub fn overlay_set_clickthrough(window: Window, ignore: bool) -> Result<bool, AppError> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| AppError::Config(format!("Failed to set cursor pass-through: {}", e)))?;
    Ok(ignore)
}

#[tauri::command]
pub fn overlay_save_layout(layout: Value) -> Result<bool, AppError> {
    let path = get_overlay_storage_dir().join("layout.json");
    if let Ok(serialized) = serde_json::to_string_pretty(&layout) {
        fs::write(path, serialized)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn overlay_load_layout() -> Result<Value, AppError> {
    let path = get_overlay_storage_dir().join("layout.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                return Ok(json);
            }
        }
    }
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub fn overlay_save_waypoints(waypoints: Vec<WaypointRecordDto>) -> Result<bool, AppError> {
    let path = get_overlay_storage_dir().join("waypoints.json");
    if let Ok(serialized) = serde_json::to_string_pretty(&waypoints) {
        fs::write(path, serialized)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn overlay_load_waypoints() -> Result<Vec<WaypointRecordDto>, AppError> {
    let path = get_overlay_storage_dir().join("waypoints.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(list) = serde_json::from_str::<Vec<WaypointRecordDto>>(&content) {
                return Ok(list);
            }
        }
    }
    Ok(Vec::new())
}

#[tauri::command]
pub fn toggle_overlay(app: AppHandle) -> bool {
    let _ = app.emit("toggleOverlay", ());
    true
}