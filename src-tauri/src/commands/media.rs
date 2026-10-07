use std::fs::{self, File};
use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Local};
use image::{imageops::FilterType, GenericImageView, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::config;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItemDto {
    pub filename: String,
    pub thumbnail: String,
    pub size_mb: f64,
    pub is_png: bool,
    pub width: u32,
    pub height: u32,
    pub date_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCompressProgressDto {
    pub current_file: String,
    pub processed_count: usize,
    pub total_files: usize,
    pub saved_mb: f64,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCompressResultDto {
    pub success: bool,
    pub compressed_count: usize,
    pub saved_mb: f64,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaBatchActionResultDto {
    pub success: bool,
    pub affected_count: usize,
    pub msg: String,
}

#[tauri::command]
pub async fn get_media_catalog(
    offset: Option<usize>,
    limit: Option<usize>,
    format_filter: Option<String>,
) -> Result<Vec<MediaItemDto>, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");

        if !screenshots_dir.exists() {
            let _ = fs::create_dir_all(&screenshots_dir);
            return Ok(Vec::new());
        }

        let off = offset.unwrap_or(0);
        let lim = limit.unwrap_or(12);
        let filter = format_filter.unwrap_or_else(|| "all".to_string()).to_lowercase();

        let mut files: Vec<_> = fs::read_dir(&screenshots_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                let is_img = name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg");
                if !is_img {
                    return false;
                }
                match filter.as_str() {
                    "png" => name.ends_with(".png"),
                    "jpg" => name.ends_with(".jpg") || name.ends_with(".jpeg"),
                    _ => true,
                }
            })
            .collect();

        files.sort_by(|a, b| {
            let a_t = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_t = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_t.cmp(&a_t)
        });

        let mut results = Vec::new();

        for entry in files.into_iter().skip(off).take(lim) {
            let file_path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            let is_png = filename.to_lowercase().ends_with(".png");

            if let Ok(metadata) = entry.metadata() {
                let size_mb = ((metadata.len() as f64) / (1024.0 * 1024.0) * 100.0).round() / 100.0;
                let date_modified = metadata
                    .modified()
                    .ok()
                    .map(|t| {
                        let dt: DateTime<Local> = t.into();
                        dt.format("%Y-%m-%d %H:%M").to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                if let Ok(reader) = ImageReader::open(&file_path) {
                    if let Ok(img) = reader.decode() {
                        let (width, height) = img.dimensions();
                        let thumbnail = img.resize(320, 180, FilterType::Nearest);
                        let mut buf = Cursor::new(Vec::new());

                        if thumbnail.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                            let b64 = BASE64.encode(buf.into_inner());
                            results.push(MediaItemDto {
                                filename,
                                thumbnail: format!("data:image/jpeg;base64,{}", b64),
                                size_mb,
                                is_png,
                                width,
                                height,
                                date_modified,
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn get_media_full_image(filename: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return Ok(String::new()),
        };

        let cfg = config::load_app_config();
        let file_path = Path::new(&cfg.current_instance).join("screenshots").join(&safe_filename);

        if !file_path.exists() {
            return Ok(String::new());
        }

        if let Ok(reader) = ImageReader::open(&file_path) {
            if let Ok(img) = reader.decode() {
                let resized = img.resize(2560, 1440, FilterType::Nearest);
                let mut buf = Cursor::new(Vec::new());
                if resized.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                    return Ok(format!("data:image/jpeg;base64,{}", BASE64.encode(buf.into_inner())));
                }
            }
        }

        Ok(String::new())
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn compress_media_stream(
    progress_channel: Channel<MediaCompressProgressDto>,
) -> Result<MediaCompressResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");

        if !screenshots_dir.exists() {
            return Ok(MediaCompressResultDto {
                success: false,
                compressed_count: 0,
                saved_mb: 0.0,
                msg: "Screenshots directory not found.".to_string(),
            });
        }

        let png_files: Vec<_> = fs::read_dir(&screenshots_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().to_lowercase().ends_with(".png"))
            .map(|e| e.path())
            .collect();

        let total_files = png_files.len();
        if total_files == 0 {
            return Ok(MediaCompressResultDto {
                success: true,
                compressed_count: 0,
                saved_mb: 0.0,
                msg: "No uncompressed PNG captures found.".to_string(),
            });
        }

        let mut saved_bytes: i64 = 0;
        let mut compressed_count = 0;

        for (idx, file_path) in png_files.iter().enumerate() {
            let filename = file_path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let new_filename = format!("{}.jpg", &filename[..filename.len() - 4]);
            let new_filepath = screenshots_dir.join(new_filename);

            if let Ok(orig_meta) = fs::metadata(&file_path) {
                let orig_size = orig_meta.len() as i64;

                if let Ok(reader) = ImageReader::open(&file_path) {
                    if let Ok(img) = reader.decode() {
                        if let Ok(mut out) = File::create(&new_filepath) {
                            if img.write_to(&mut out, ImageFormat::Jpeg).is_ok() {
                                if let Ok(new_meta) = fs::metadata(&new_filepath) {
                                    let new_size = new_meta.len() as i64;
                                    if new_size < orig_size {
                                        saved_bytes += orig_size - new_size;
                                        let _ = fs::remove_file(&file_path);
                                        compressed_count += 1;
                                    } else {
                                        let _ = fs::remove_file(&new_filepath);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let percent = ((idx + 1) as f64 / total_files as f64) * 100.0;
            let saved_mb = ((saved_bytes as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0;

            let _ = progress_channel.send(MediaCompressProgressDto {
                current_file: filename,
                processed_count: idx + 1,
                total_files,
                saved_mb,
                percent,
            });
        }

        let total_saved_mb = ((saved_bytes as f64 / (1024.0 * 1024.0)) * 10.0).round() / 10.0;

        Ok(MediaCompressResultDto {
            success: true,
            compressed_count,
            saved_mb: total_saved_mb,
            msg: format!("Compressed {} captures, freeing {:.1} MB.", compressed_count, total_saved_mb),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn delete_media_file(filename: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let safe_filename = match Path::new(&filename).file_name() {
            Some(n) => n.to_string_lossy().to_string(),
            None => return Ok(false),
        };

        let cfg = config::load_app_config();
        let file_path = Path::new(&cfg.current_instance).join("screenshots").join(safe_filename);

        if file_path.exists() {
            fs::remove_file(file_path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub async fn batch_delete_media_files(filenames: Vec<String>) -> Result<MediaBatchActionResultDto, AppError> {
    tokio::task::spawn_blocking(move || {
        let cfg = config::load_app_config();
        let screenshots_dir = Path::new(&cfg.current_instance).join("screenshots");
        let mut deleted_count = 0;

        for name in filenames {
            if let Some(safe_name) = Path::new(&name).file_name() {
                let p = screenshots_dir.join(safe_name);
                if p.exists() && fs::remove_file(p).is_ok() {
                    deleted_count += 1;
                }
            }
        }

        Ok(MediaBatchActionResultDto {
            success: true,
            affected_count: deleted_count,
            msg: format!("Purged {} capture files from storage.", deleted_count),
        })
    })
    .await
    .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[tauri::command]
pub fn open_screenshots_directory() -> Result<(), AppError> {
    let cfg = config::load_app_config();
    let dir = Path::new(&cfg.current_instance).join("screenshots");
    let _ = fs::create_dir_all(&dir);

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(dir).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(dir).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
    Ok(())
}