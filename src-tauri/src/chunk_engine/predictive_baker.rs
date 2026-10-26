use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::chunk_engine::checkerboard_scheduler::{CheckerboardScheduler, ChunkCoordinate};
use crate::chunk_engine::mca_ring_io::{ChunkWritePayload, McaRingIoEngine};
use crate::chunk_engine::offheap_arena::OffHeapVoxelArena;
use crate::chunk_engine::simd_noise::SimdNoiseEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkPrebakeProgressDto {
    pub current_chunk: String,
    pub generated_count: usize,
    pub total_target_chunks: usize,
    pub percent: f64,
    pub speed_chunks_per_sec: f64,
    pub active_workers: usize,
    pub throughput_mb_per_sec: f64,
    pub eta_seconds: u64,
    pub grid_size: usize,
    pub grid_cells: Vec<u8>,
    pub recent_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkPrebakeResultDto {
    pub success: bool,
    pub generated_chunks: usize,
    pub elapsed_ms: u64,
    pub average_throughput: f64,
    pub total_bytes_mb: f64,
    pub message: String,
}

pub struct PredictiveChunkBaker;

impl PredictiveChunkBaker {
    fn build_minecraft_chunk_nbt(cx: i32, cz: i32, densities: &[f32]) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(3072);
        buffer.push(10);
        buffer.extend_from_slice(b"\x00\x00");

        buffer.push(3);
        buffer.extend_from_slice(b"\x00\x0BDataVersion");
        buffer.extend_from_slice(&3955i32.to_be_bytes());

        buffer.push(3);
        buffer.extend_from_slice(b"\x00\x04xPos");
        buffer.extend_from_slice(&cx.to_be_bytes());

        buffer.push(3);
        buffer.extend_from_slice(b"\x00\x04zPos");
        buffer.extend_from_slice(&cz.to_be_bytes());

        buffer.push(3);
        buffer.extend_from_slice(b"\x00\x04yPos");
        buffer.extend_from_slice(&(-4i32).to_be_bytes());

        buffer.push(8);
        buffer.extend_from_slice(b"\x00\x06Status");
        buffer.extend_from_slice(b"\x00\x09minecraft:full");

        buffer.push(9);
        buffer.extend_from_slice(b"\x00\x08sections");
        buffer.push(10);
        buffer.extend_from_slice(&24i32.to_be_bytes());

        for sec_idx in -4i8..20i8 {
            buffer.push(1);
            buffer.extend_from_slice(b"\x00\x01Y");
            buffer.push(sec_idx as u8);

            buffer.push(10);
            buffer.extend_from_slice(b"\x00\x0Cblock_states");

            buffer.push(9);
            buffer.extend_from_slice(b"\x00\x07palette");
            buffer.push(10);
            buffer.extend_from_slice(&1i32.to_be_bytes());

            let sample_index = ((sec_idx + 4) as usize * 16).min(densities.len().saturating_sub(1));
            let density_sample = densities.get(sample_index).copied().unwrap_or(0.0);

            buffer.push(8);
            buffer.extend_from_slice(b"\x00\x04Name");
            if density_sample > 0.0 {
                if sec_idx < 0 {
                    buffer.extend_from_slice(b"\x00\x13minecraft:deepslate");
                } else if sec_idx == 4 {
                    buffer.extend_from_slice(b"\x00\x15minecraft:grass_block");
                } else {
                    buffer.extend_from_slice(b"\x00\x0Fminecraft:stone");
                }
            } else {
                buffer.extend_from_slice(b"\x00\x0Dminecraft:air");
            }
            buffer.push(0);
            buffer.push(0);

            buffer.push(10);
            buffer.extend_from_slice(b"\x00\x06biomes");
            buffer.push(9);
            buffer.extend_from_slice(b"\x00\x07palette");
            buffer.push(8);
            buffer.extend_from_slice(&1i32.to_be_bytes());
            buffer.extend_from_slice(b"\x00\x10minecraft:plains");
            buffer.push(0);

            buffer.push(0);
        }

        buffer.push(0);
        buffer
    }

    pub fn bake_directional_cone(
        instance_path: &Path,
        center_x: i32,
        center_z: i32,
        radius_chunks: i32,
        scheduler: &CheckerboardScheduler,
        arena: &OffHeapVoxelArena,
        ring_io: &McaRingIoEngine,
        progress_channel: Channel<ChunkPrebakeProgressDto>,
    ) -> Result<ChunkPrebakeResultDto, String> {
        let start = Instant::now();
        let rad = radius_chunks.clamp(4, 32);
        let side = (rad * 2 + 1) as usize;
        let total_target = side * side;

        scheduler.reset();

        for dz in -rad..=rad {
            for dx in -rad..=rad {
                scheduler.schedule_coordinate(ChunkCoordinate {
                    x: center_x + dx,
                    z: center_z + dz,
                });
            }
        }

        let mut grid_cells = vec![0u8; total_target];
        let mut recent_events: Vec<String> = Vec::new();

        let num_cpus = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(2, 32);

        let generated_counter = Arc::new(AtomicUsize::new(0));
        let total_payload_bytes = Arc::new(AtomicUsize::new(0));
        let mut last_report = Instant::now();
        let mut empty_streak = 0;

        recent_events.push(format!(
            "Voxel Engine worker pool initialized ({} hardware threads)",
            num_cpus
        ));

        while generated_counter.load(Ordering::Relaxed) < total_target && empty_streak < 4 {
            let (_, batch) = scheduler.pop_batch_for_phase(num_cpus * 16);
            if batch.is_empty() {
                empty_streak += 1;
                thread::yield_now();
                continue;
            }
            empty_streak = 0;

            for coord in &batch {
                let gx = (coord.x - (center_x - rad)) as usize;
                let gz = (coord.z - (center_z - rad)) as usize;
                if gx < side && gz < side {
                    grid_cells[gz * side + gx] = 1;
                }
            }

            let chunks_per_worker = (batch.len() + num_cpus - 1) / num_cpus;
            let chunks_slices: Vec<Vec<ChunkCoordinate>> = batch
                .chunks(chunks_per_worker)
                .map(|s| s.to_vec())
                .collect();

            let mut processed_payloads: Vec<ChunkWritePayload> = Vec::with_capacity(batch.len());

            thread::scope(|s| {
                let mut handles = Vec::new();
                for slice in chunks_slices {
                    handles.push(s.spawn(move || {
                        let mut local_results = Vec::with_capacity(slice.len());
                        for coord in slice {
                            let _slab = arena.allocate_chunk_slab(4096);
                            let densities = SimdNoiseEngine::sample_voxel_column_simd(coord.x, coord.z, 384);
                            let nbt = Self::build_minecraft_chunk_nbt(coord.x, coord.z, &densities);
                            arena.release_chunk_slab();

                            let rx = coord.x >> 5;
                            let rz = coord.z >> 5;

                            local_results.push(ChunkWritePayload {
                                region_x: rx,
                                region_z: rz,
                                chunk_x: coord.x,
                                chunk_z: coord.z,
                                raw_nbt: nbt,
                            });
                        }
                        local_results
                    }));
                }

                for handle in handles {
                    if let Ok(results) = handle.join() {
                        processed_payloads.extend(results);
                    }
                }
            });

            for payload in processed_payloads {
                let cx = payload.chunk_x;
                let cz = payload.chunk_z;
                total_payload_bytes.fetch_add(payload.raw_nbt.len(), Ordering::Relaxed);

                ring_io.push_chunk(payload);
                let current_gen = generated_counter.fetch_add(1, Ordering::Relaxed) + 1;

                let gx = (cx - (center_x - rad)) as usize;
                let gz = (cz - (center_z - rad)) as usize;
                if gx < side && gz < side {
                    grid_cells[gz * side + gx] = 2;
                }

                if current_gen % 128 == 0 {
                    let rx = cx >> 5;
                    let rz = cz >> 5;
                    recent_events.push(format!(
                        "Buffered chunk [{}, {}] into r.{}.{}.mca",
                        cx, cz, rx, rz
                    ));
                    if recent_events.len() > 8 {
                        recent_events.remove(0);
                    }
                }

                if last_report.elapsed().as_millis() >= 40 || current_gen == total_target {
                    let elapsed_secs = start.elapsed().as_secs_f64().max(0.001);
                    let speed = (current_gen as f64) / elapsed_secs;
                    let throughput = ((total_payload_bytes.load(Ordering::Relaxed) as f64)
                        / (1024.0 * 1024.0))
                        / elapsed_secs;
                    let remaining_chunks = total_target.saturating_sub(current_gen);
                    let eta = if speed > 0.0 {
                        (remaining_chunks as f64 / speed).ceil() as u64
                    } else {
                        0
                    };

                    let pct = ((current_gen as f64) / (total_target as f64)) * 100.0;

                    let _ = progress_channel.send(ChunkPrebakeProgressDto {
                        current_chunk: format!("[{}, {}]", cx, cz),
                        generated_count: current_gen,
                        total_target_chunks: total_target,
                        percent: (pct * 10.0).round() / 10.0,
                        speed_chunks_per_sec: (speed * 10.0).round() / 10.0,
                        active_workers: num_cpus,
                        throughput_mb_per_sec: (throughput * 10.0).round() / 10.0,
                        eta_seconds: eta,
                        grid_size: side,
                        grid_cells: grid_cells.clone(),
                        recent_events: recent_events.clone(),
                    });
                    last_report = Instant::now();
                }
            }
        }

        recent_events.push("Flushing ring buffer to region archives...".to_string());
        let (saved, bytes) = ring_io.flush_batch(instance_path);
        arena.reset();

        for cell in &mut grid_cells {
            if *cell == 2 {
                *cell = 3;
            }
        }

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let final_count = generated_counter.load(Ordering::Relaxed);
        let final_speed = (final_count as f64) / (start.elapsed().as_secs_f64().max(0.001));
        let total_mb = (bytes as f64) / (1024.0 * 1024.0);

        let _ = progress_channel.send(ChunkPrebakeProgressDto {
            current_chunk: "Completed".to_string(),
            generated_count: final_count,
            total_target_chunks: total_target,
            percent: 100.0,
            speed_chunks_per_sec: (final_speed * 10.0).round() / 10.0,
            active_workers: 0,
            throughput_mb_per_sec: 0.0,
            eta_seconds: 0,
            grid_size: side,
            grid_cells,
            recent_events: vec![format!("Committed {} chunks to disk successfully", saved)],
        });

        Ok(ChunkPrebakeResultDto {
            success: true,
            generated_chunks: final_count,
            elapsed_ms,
            average_throughput: (final_speed * 10.0).round() / 10.0,
            total_bytes_mb: (total_mb * 10.0).round() / 10.0,
            message: format!(
                "Synthesized {} chunks using {} threads in {} ms ({:.1} chunks/sec, {:.2} MB)",
                final_count, num_cpus, elapsed_ms, final_speed, total_mb
            ),
        })
    }
}