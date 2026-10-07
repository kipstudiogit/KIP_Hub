use std::path::Path;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChunkPrebakeResultDto {
    pub success: bool,
    pub generated_chunks: usize,
    pub elapsed_ms: u64,
    pub average_throughput: f64,
    pub message: String,
}

pub struct PredictiveChunkBaker;

impl PredictiveChunkBaker {
    fn build_minecraft_chunk_nbt(cx: i32, cz: i32, densities: &[f32], _arena_ptr: *mut u8, _arena_size: usize) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(4096);
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
        let total_target = (((rad * 2 + 1) * (rad * 2 + 1)) as usize).max(1);

        scheduler.reset();

        for dx in -rad..=rad {
            for dz in -rad..=rad {
                scheduler.schedule_coordinate(ChunkCoordinate {
                    x: center_x + dx,
                    z: center_z + dz,
                });
            }
        }

        let mut generated = 0;
        let mut last_report = Instant::now();
        let mut empty_streak = 0;

        while generated < total_target && empty_streak < 4 {
            let (_, batch) = scheduler.pop_batch_for_phase(64);
            if batch.is_empty() {
                empty_streak += 1;
                continue;
            }
            empty_streak = 0;

            for coord in batch {
                let densities = SimdNoiseEngine::sample_voxel_column_simd(coord.x, coord.z, 384);
                let slab_size = 65536;

                let nbt_payload = if let Some(slab_ptr) = arena.allocate_chunk_slab(slab_size) {
                    let nbt = Self::build_minecraft_chunk_nbt(coord.x, coord.z, &densities, slab_ptr, slab_size);
                    arena.release_chunk_slab();
                    nbt
                } else {
                    Self::build_minecraft_chunk_nbt(coord.x, coord.z, &densities, std::ptr::null_mut(), 0)
                };

                let rx = coord.x >> 5;
                let rz = coord.z >> 5;

                ring_io.push_chunk(ChunkWritePayload {
                    region_x: rx,
                    region_z: rz,
                    chunk_x: coord.x,
                    chunk_z: coord.z,
                    raw_nbt: nbt_payload,
                });

                generated += 1;

                if last_report.elapsed().as_millis() >= 60 || generated == total_target {
                    let elapsed_secs = start.elapsed().as_secs_f64().max(0.001);
                    let speed = (generated as f64) / elapsed_secs;
                    let pct = ((generated as f64) / (total_target as f64)) * 100.0;

                    let _ = progress_channel.send(ChunkPrebakeProgressDto {
                        current_chunk: format!("[{}, {}]", coord.x, coord.z),
                        generated_count: generated,
                        total_target_chunks: total_target,
                        percent: (pct * 10.0).round() / 10.0,
                        speed_chunks_per_sec: (speed * 10.0).round() / 10.0,
                    });
                    last_report = Instant::now();
                }
            }
        }

        ring_io.flush_batch(instance_path);
        arena.reset();

        let elapsed_ms = start.elapsed().as_millis() as u64;
        let final_speed = (generated as f64) / (start.elapsed().as_secs_f64().max(0.001));

        Ok(ChunkPrebakeResultDto {
            success: true,
            generated_chunks: generated,
            elapsed_ms,
            average_throughput: (final_speed * 10.0).round() / 10.0,
            message: format!(
                "Synthesized {} chunks across 4 checkerboard phases in {} ms ({:.1} chunks/sec)",
                generated, elapsed_ms, final_speed
            ),
        })
    }
}