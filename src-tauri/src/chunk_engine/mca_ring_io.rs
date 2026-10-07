use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use parking_lot::Mutex;

pub struct ChunkWritePayload {
    pub region_x: i32,
    pub region_z: i32,
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub raw_nbt: Vec<u8>,
}

pub struct McaRingIoEngine {
    queue: Arc<Mutex<VecDeque<ChunkWritePayload>>>,
    is_flushing: Arc<AtomicBool>,
    total_chunks_saved: Arc<AtomicU64>,
    total_bytes_written: Arc<AtomicU64>,
}

impl McaRingIoEngine {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(1024))),
            is_flushing: Arc::new(AtomicBool::new(false)),
            total_chunks_saved: Arc::new(AtomicU64::new(0)),
            total_bytes_written: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn push_chunk(&self, payload: ChunkWritePayload) {
        let mut guard = self.queue.lock();
        if guard.len() >= 8192 {
            guard.pop_front();
        }
        guard.push_back(payload);
    }

    pub fn resolve_active_region_dir(instance_path: &Path) -> PathBuf {
        let saves_dir = instance_path.join("saves");
        if saves_dir.exists() {
            if let Ok(entries) = fs::read_dir(&saves_dir) {
                let mut latest_dir: Option<PathBuf> = None;
                let mut latest_time = SystemTime::UNIX_EPOCH;

                for entry in entries.filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_dir() && p.join("level.dat").exists() {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(mod_time) = meta.modified() {
                                if mod_time > latest_time {
                                    latest_time = mod_time;
                                    latest_dir = Some(p);
                                }
                            }
                        }
                    }
                }

                if let Some(w) = latest_dir {
                    let r_dir = w.join("region");
                    let _ = fs::create_dir_all(&r_dir);
                    return r_dir;
                }
            }
        }

        let fallback_region = saves_dir.join("Quantum_World").join("region");
        let _ = fs::create_dir_all(&fallback_region);
        fallback_region
    }

    pub fn flush_batch(&self, instance_path: &Path) -> (usize, u64) {
        if self.is_flushing.swap(true, Ordering::SeqCst) {
            return (0, 0);
        }

        let batches: Vec<ChunkWritePayload> = {
            let mut guard = self.queue.lock();
            guard.drain(..).collect()
        };

        if batches.is_empty() {
            self.is_flushing.store(false, Ordering::SeqCst);
            return (0, 0);
        }

        let region_dir = Self::resolve_active_region_dir(instance_path);
        let mut region_groups: HashMap<(i32, i32), Vec<ChunkWritePayload>> = HashMap::new();

        for item in batches {
            region_groups
                .entry((item.region_x, item.region_z))
                .or_default()
                .push(item);
        }

        let mut saved_count = 0;
        let mut bytes_count = 0u64;

        for ((rx, rz), chunks) in region_groups {
            let mca_name = format!("r.{}.{}.mca", rx, rz);
            let mca_path = region_dir.join(mca_name);

            let file_res = OpenOptions::new()
                .create(true)
                .read(true)
                .write(true)
                .open(&mca_path);

            let mut file: File = match file_res {
                Ok(f) => f,
                Err(_) => continue,
            };

            let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);
            if current_len < 8192 {
                let initial_header = vec![0u8; 8192];
                if file.seek(SeekFrom::Start(0)).is_err() || file.write_all(&initial_header).is_err() {
                    continue;
                }
                let _ = file.flush();
            }

            for item in chunks {
                let mut compressed = Vec::new();
                let mut encoder = ZlibEncoder::new(&mut compressed, Compression::fast());
                if encoder.write_all(&item.raw_nbt).is_err() || encoder.finish().is_err() {
                    continue;
                }

                let rx_local = ((item.chunk_x % 32) + 32) % 32;
                let rz_local = ((item.chunk_z % 32) + 32) % 32;
                let table_offset = ((rx_local + rz_local * 32) * 4) as u64;

                let sector_len = ((compressed.len() + 5 + 4095) / 4096) as u8;
                let actual_file_len = file.metadata().map(|m| m.len()).unwrap_or(8192).max(8192);
                let target_sector = (actual_file_len + 4095) / 4096;

                if file.seek(SeekFrom::Start(table_offset)).is_ok() {
                    let b0 = ((target_sector >> 16) & 0xFF) as u8;
                    let b1 = ((target_sector >> 8) & 0xFF) as u8;
                    let b2 = (target_sector & 0xFF) as u8;
                    let b3 = sector_len;
                    let _ = file.write_all(&[b0, b1, b2, b3]);
                }

                let now_unix = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs() as u32)
                    .unwrap_or(0);

                if file.seek(SeekFrom::Start(table_offset + 4096)).is_ok() {
                    let _ = file.write_all(&now_unix.to_be_bytes());
                }

                if file.seek(SeekFrom::Start(target_sector * 4096)).is_ok() {
                    let len_bytes = ((compressed.len() + 1) as u32).to_be_bytes();
                    if file.write_all(&len_bytes).is_ok()
                        && file.write_all(&[2u8]).is_ok()
                        && file.write_all(&compressed).is_ok()
                    {
                        let padding = ((sector_len as usize) * 4096).saturating_sub(compressed.len() + 5);
                        if padding > 0 {
                            let pad_bytes = vec![0u8; padding];
                            let _ = file.write_all(&pad_bytes);
                        }
                        let _ = file.flush();
                        saved_count += 1;
                        bytes_count += compressed.len() as u64;
                    }
                }
            }
        }

        self.total_chunks_saved.fetch_add(saved_count as u64, Ordering::Relaxed);
        self.total_bytes_written.fetch_add(bytes_count, Ordering::Relaxed);
        self.is_flushing.store(false, Ordering::SeqCst);

        (saved_count, bytes_count)
    }

    pub fn get_metrics(&self) -> (u64, u64, usize) {
        (
            self.total_chunks_saved.load(Ordering::Relaxed),
            self.total_bytes_written.load(Ordering::Relaxed),
            self.queue.lock().len(),
        )
    }
}