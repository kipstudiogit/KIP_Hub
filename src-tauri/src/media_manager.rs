use std::fs::{self, File};
use std::path::Path;
use std::io::Cursor;
use std::process::Command;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::{json, Value};
use image::{ImageFormat, ImageReader, imageops::FilterType};

pub struct MediaManager;

impl MediaManager {
    pub fn get_media(screenshots_dir: &str, offset: usize, limit: usize) -> Vec<Value> {
        let mut results = Vec::new();
        let path = Path::new(screenshots_dir);

        if !path.exists() {
            return results;
        }

        let mut files: Vec<_> = fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg")
            })
            .collect();

        files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_time.cmp(&a_time)
        });

        for entry in files.into_iter().skip(offset).take(limit) {
            let file_path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            let is_png = filename.to_lowercase().ends_with(".png");

            if let Ok(metadata) = entry.metadata() {
                let size_mb = (metadata.len() as f64) / (1024.0 * 1024.0);
                let size_mb = (size_mb * 100.0).round() / 100.0;

                if let Ok(reader) = ImageReader::open(&file_path) {
                    if let Ok(img) = reader.decode() {
                        let thumbnail = img.resize(256, 144, FilterType::Nearest);
                        let mut buf = Cursor::new(Vec::new());
                        if thumbnail.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                            let b64 = BASE64.encode(buf.into_inner());
                            let thumbnail_b64 = format!("data:image/jpeg;base64,{}", b64);

                            results.push(json!({
                                "filename": filename,
                                "size": size_mb,
                                "thumbnail": thumbnail_b64,
                                "is_png": is_png
                            }));
                        }
                    }
                }
            }
        }

        results
    }

    pub fn get_media_full(screenshots_dir: &str, filename: &str) -> String {
        let file_path = Path::new(screenshots_dir).join(filename);
        if !file_path.exists() {
            return String::new();
        }

        if let Ok(reader) = ImageReader::open(&file_path) {
            if let Ok(img) = reader.decode() {
                let resized = img.resize(1920, 1080, FilterType::Nearest);
                let mut buf = Cursor::new(Vec::new());
                if resized.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                    let b64 = BASE64.encode(buf.into_inner());
                    return format!("data:image/jpeg;base64,{}", b64);
                }
            }
        }
        String::new()
    }

    pub fn compress_media(screenshots_dir: &str) -> Value {
        let path = Path::new(screenshots_dir);
        if !path.exists() {
            return json!({ "success": false, "msg": "Screenshots directory not found." });
        }

        let mut saved_bytes: i64 = 0;
        let mut compressed_count = 0;

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let file_path = entry.path();
                let filename = entry.file_name().to_string_lossy().to_string();

                if filename.to_lowercase().ends_with(".png") {
                    let new_filename = format!("{}.jpg", &filename[..filename.len() - 4]);
                    let new_filepath = path.join(new_filename);

                    if let Ok(orig_metadata) = fs::metadata(&file_path) {
                        let orig_size = orig_metadata.len() as i64;

                        if let Ok(reader) = ImageReader::open(&file_path) {
                            if let Ok(img) = reader.decode() {
                                if let Ok(mut buf) = File::create(&new_filepath) {
                                    if img.write_to(&mut buf, ImageFormat::Jpeg).is_ok() {
                                        if let Ok(new_metadata) = fs::metadata(&new_filepath) {
                                            let new_size = new_metadata.len() as i64;
                                            saved_bytes += orig_size - new_size;
                                            let _ = fs::remove_file(&file_path);
                                            compressed_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if compressed_count == 0 {
            return json!({ "success": false, "msg": "No PNG files to compress." });
        }

        let saved_mb = (saved_bytes as f64) / (1024.0 * 1024.0);
        let saved_mb = (saved_mb * 100.0).round() / 100.0;

        json!({
            "success": true,
            "msg": format!("Compressed {} images. Saved {} MB!", compressed_count, saved_mb)
        })
    }

    pub fn delete_media(screenshots_dir: &str, filename: &str) -> bool {
        let file_path = Path::new(screenshots_dir).join(filename);
        if file_path.exists() {
            fs::remove_file(file_path).is_ok()
        } else {
            false
        }
    }

    pub fn open_folder(screenshots_dir: &str) {
        let path = Path::new(screenshots_dir);
        if !path.exists() {
            let _ = fs::create_dir_all(path);
        }

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("explorer").arg(screenshots_dir).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = Command::new("open").arg(screenshots_dir).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            let _ = Command::new("xdg-open").arg(screenshots_dir).spawn();
        }
    }
}