use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
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
            queue: Arc::new(Mutex::new(VecDeque::with_capacity(4096))),
            is_flushing: Arc::new(AtomicBool::new(false)),
            total_chunks_saved: Arc::new(AtomicU64::new(0)),
            total_bytes_written: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn push_chunk(&self, payload: ChunkWritePayload) {
        let mut guard = self.queue.lock();
        if guard.len() >= 16384 {
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

        let fallback_region = saves_dir.join("Voxel_World").join("region");
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

            let mut header = vec![0u8; 8192];
            let current_len = file.metadata().map(|m| m.len()).unwrap_or(0);

            if current_len >= 8192 {
                if file.seek(SeekFrom::Start(0)).is_ok() {
                    let _ = file.read_exact(&mut header);
                }
            } else {
                if file.seek(SeekFrom::Start(0)).is_err() || file.write_all(&header).is_err() {
                    continue;
                }
                let _ = file.flush();
            }

            let mut max_sector = 2u64;
            for i in 0..1024 {
                let offset_idx = i * 4;
                let b0 = header[offset_idx] as u64;
                let b1 = header[offset_idx + 1] as u64;
                let b2 = header[offset_idx + 2] as u64;
                let sector_count = header[offset_idx + 3] as u64;
                let sector_offset = (b0 << 16) | (b1 << 8) | b2;
                if sector_offset > 0 && sector_count > 0 {
                    let end_sec = sector_offset + sector_count;
                    if end_sec > max_sector {
                        max_sector = end_sec;
                    }
                }
            }

            let now_unix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0);

            for item in chunks {
                let mut compressed = Vec::new();
                let mut encoder = ZlibEncoder::new(&mut compressed, Compression::fast());
                if encoder.write_all(&item.raw_nbt).is_err() || encoder.finish().is_err() {
                    continue;
                }

                let rx_local = ((item.chunk_x.rem_euclid(32)) + 32) % 32;
                let rz_local = ((item.chunk_z.rem_euclid(32)) + 32) % 32;
                let chunk_idx = (rx_local + rz_local * 32) as usize;
                let offset_idx = chunk_idx * 4;
                let ts_idx = 4096 + chunk_idx * 4;

                let payload_len = (compressed.len() + 5) as u64;
                let sector_len = ((payload_len + 4095) / 4096) as u8;

                let target_sector = max_sector;
                max_sector += sector_len as u64;

                if file.seek(SeekFrom::Start(target_sector * 4096)).is_err() {
                    continue;
                }

                let len_bytes = ((compressed.len() + 1) as u32).to_be_bytes();
                if file.write_all(&len_bytes).is_err()
                    || file.write_all(&[2u8]).is_err()
                    || file.write_all(&compressed).is_err()
                {
                    continue;
                }

                let padding = ((sector_len as usize) * 4096).saturating_sub(compressed.len() + 5);
                if padding > 0 {
                    let pad = vec![0u8; padding];
                    let _ = file.write_all(&pad);
                }

                header[offset_idx] = ((target_sector >> 16) & 0xFF) as u8;
                header[offset_idx + 1] = ((target_sector >> 8) & 0xFF) as u8;
                header[offset_idx + 2] = (target_sector & 0xFF) as u8;
                header[offset_idx + 3] = sector_len;

                let ts_bytes = now_unix.to_be_bytes();
                header[ts_idx..ts_idx + 4].copy_from_slice(&ts_bytes);

                saved_count += 1;
                bytes_count += payload_len;
            }

            if file.seek(SeekFrom::Start(0)).is_ok() {
                let _ = file.write_all(&header);
                let _ = file.flush();
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