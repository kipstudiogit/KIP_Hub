use serde::{Deserialize, Serialize};
use tauri::Window;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowStatusDto {
    pub is_maximized: bool,
    pub is_minimized: bool,
    pub is_fullscreen: bool,
    pub is_always_on_top: bool,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiniModeResultDto {
    pub is_mini: bool,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub fn window_get_status(window: Window) -> Result<WindowStatusDto, AppError> {
    let is_maximized = window.is_maximized().unwrap_or(false);
    let is_minimized = window.is_minimized().unwrap_or(false);
    let is_fullscreen = window.is_fullscreen().unwrap_or(false);
    let scale_factor = window.scale_factor().unwrap_or(1.0);

    let inner_size = window
        .inner_size()
        .map(|s| s.to_logical::<f64>(scale_factor))
        .unwrap_or(tauri::LogicalSize {
            width: 1280.0,
            height: 850.0,
        });

    Ok(WindowStatusDto {
        is_maximized,
        is_minimized,
        is_fullscreen,
        is_always_on_top: false,
        width: inner_size.width,
        height: inner_size.height,
        scale_factor,
    })
}

#[tauri::command]
pub fn window_minimize(window: Window) -> Result<(), AppError> {
    window
        .minimize()
        .map_err(|e| AppError::Config(format!("Failed to minimize window: {}", e)))
}

#[tauri::command]
pub fn window_toggle_maximize(window: Window) -> Result<bool, AppError> {
    let is_max = window.is_maximized().unwrap_or(false);
    if is_max {
        window
            .unmaximize()
            .map_err(|e| AppError::Config(format!("Failed to unmaximize window: {}", e)))?;
        Ok(false)
    } else {
        window
            .maximize()
            .map_err(|e| AppError::Config(format!("Failed to maximize window: {}", e)))?;
        Ok(true)
    }
}

#[tauri::command]
pub fn window_maximize(window: Window) -> Result<bool, AppError> {
    window_toggle_maximize(window)
}

#[tauri::command]
pub fn window_is_maximized(window: Window) -> bool {
    window.is_maximized().unwrap_or(false)
}

#[tauri::command]
pub fn window_close(window: Window) -> Result<(), AppError> {
    window
        .close()
        .map_err(|e| AppError::Config(format!("Failed to close window: {}", e)))
}

#[tauri::command]
pub fn window_toggle_pin(window: Window, current_pinned: bool) -> Result<bool, AppError> {
    let new_state = !current_pinned;
    window
        .set_always_on_top(new_state)
        .map_err(|e| AppError::Config(format!("Failed to toggle always on top: {}", e)))?;
    Ok(new_state)
}

#[tauri::command]
pub fn window_set_mini_mode(window: Window, mini: bool) -> Result<MiniModeResultDto, AppError> {
    if mini {
        let _ = window.set_resizable(false);
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 380.0,
            height: 590.0,
        }));
    } else {
        let _ = window.set_resizable(true);
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 1320.0,
            height: 880.0,
        }));
        let _ = window.center();
    }

    Ok(MiniModeResultDto {
        is_mini: mini,
        width: if mini { 380.0 } else { 1320.0 },
        height: if mini { 590.0 } else { 880.0 },
    })
}

#[tauri::command]
pub fn set_mini_mode(window: Window, mini: bool) -> Result<MiniModeResultDto, AppError> {
    window_set_mini_mode(window, mini)
}

#[tauri::command]
pub fn window_toggle_fullscreen(window: Window) -> Result<bool, AppError> {
    let is_full = window.is_fullscreen().unwrap_or(false);
    let next = !is_full;
    window
        .set_fullscreen(next)
        .map_err(|e| AppError::Config(format!("Failed to set fullscreen mode: {}", e)))?;
    Ok(next)
}

#[tauri::command]
pub fn toggle_big_picture(window: Window) -> bool {
    window_toggle_fullscreen(window).unwrap_or(false)
}