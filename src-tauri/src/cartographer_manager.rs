use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Cursor};
use std::path::Path;
use std::collections::HashMap;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use flate2::read::ZlibDecoder;
use fastnbt::Value as NbtValue;
use image::{RgbaImage, Rgba, ImageFormat, imageops::FilterType};

pub struct CartographerManager {
    color_map: HashMap<&'static str, [u8; 4]>,
    ignored_blocks: Vec<&'static str>,
}

impl CartographerManager {
    pub fn new() -> Self {
        let mut color_map = HashMap::new();
        color_map.insert("minecraft:water", [35, 137, 218, 255]);
        color_map.insert("minecraft:lava", [207, 85, 22, 255]);
        color_map.insert("minecraft:grass_block", [89, 166, 68, 255]);
        color_map.insert("minecraft:dirt", [134, 96, 67, 255]);
        color_map.insert("minecraft:coarse_dirt", [119, 85, 59, 255]);
        color_map.insert("minecraft:podzol", [90, 63, 28, 255]);
        color_map.insert("minecraft:mycelium", [111, 99, 105, 255]);
        color_map.insert("minecraft:sand", [219, 211, 160, 255]);
        color_map.insert("minecraft:red_sand", [169, 88, 33, 255]);
        color_map.insert("minecraft:stone", [125, 125, 125, 255]);
        color_map.insert("minecraft:deepslate", [77, 77, 80, 255]);
        color_map.insert("minecraft:cobblestone", [100, 100, 100, 255]);
        color_map.insert("minecraft:bedrock", [83, 83, 83, 255]);
        color_map.insert("minecraft:snow", [255, 255, 255, 255]);
        color_map.insert("minecraft:snow_block", [255, 255, 255, 255]);
        color_map.insert("minecraft:ice", [148, 168, 253, 255]);
        color_map.insert("minecraft:packed_ice", [141, 180, 250, 255]);
        color_map.insert("minecraft:blue_ice", [116, 167, 253, 255]);
        color_map.insert("minecraft:gravel", [128, 124, 117, 255]);
        color_map.insert("minecraft:clay", [160, 166, 179, 255]);
        color_map.insert("minecraft:netherrack", [112, 2, 0, 255]);
        color_map.insert("minecraft:end_stone", [223, 224, 165, 255]);
        color_map.insert("minecraft:obsidian", [20, 18, 29, 255]);

        let ignored_blocks = vec![
            "minecraft:air",
            "minecraft:cave_air",
            "minecraft:void_air",
            "minecraft:barrier",
            "minecraft:light",
            "minecraft:structure_void",
        ];

        Self {
            color_map,
            ignored_blocks,
        }
    }

    fn guess_color(&self, block_id: &str) -> [u8; 4] {
        let b = block_id.to_lowercase();
        if b.contains("leaves") { return [72, 181, 72, 255]; }
        if b.contains("log") || b.contains("planks") || b.contains("wood") { return [143, 119, 72, 255]; }
        if b.contains("stone") || b.contains("andesite") || b.contains("diorite") { return [130, 130, 130, 255]; }
        if b.contains("sand") { return [219, 211, 160, 255]; }
        if b.contains("dirt") { return [134, 96, 67, 255]; }
        if b.contains("glass") { return [175, 214, 222, 150]; }
        if b.contains("terracotta") { return [152, 94, 67, 255]; }
        if b.contains("concrete") { return [127, 131, 134, 255]; }
        if b.contains("wool") { return [225, 225, 225, 255]; }
        [100, 100, 100, 255]
    }

    pub fn generate_map(&self, saves_dir: &str, world_name: &str, radius: i32) -> String {
        let region_dir = Path::new(saves_dir).join(world_name).join("region");
        if !region_dir.exists() {
            return String::new();
        }

        let map_dim = ((radius * 2 + 1) * 512) as u32;
        let mut img = RgbaImage::new(map_dim, map_dim);

        for rx in -radius..=radius {
            for rz in -radius..=radius {
                let mca_path = region_dir.join(format!("r.{}.{}.mca", rx, rz));
                if !mca_path.exists() {
                    continue;
                }

                let mut file = match File::open(&mca_path) {
                    Ok(f) => f,
                    Err(_) => continue,
                };

                let mut header = [0u8; 4096];
                if file.read_exact(&mut header).is_err() {
                    continue;
                }

                for cx in 0..32 {
                    for cz in 0..32 {
                        let offset_index = ((cx & 31) + (cz & 31) * 32) * 4;
                        let b0 = header[offset_index] as u64;
                        let b1 = header[offset_index + 1] as u64;
                        let b2 = header[offset_index + 2] as u64;
                        let sectors = header[offset_index + 3];

                        if sectors == 0 {
                            continue;
                        }

                        let sector_offset = (b0 << 16) | (b1 << 8) | b2;
                        if file.seek(SeekFrom::Start(sector_offset * 4096)).is_err() {
                            continue;
                        }

                        let mut chunk_len_bytes = [0u8; 4];
                        if file.read_exact(&mut chunk_len_bytes).is_err() {
                            continue;
                        }
                        let chunk_len = u32::from_be_bytes(chunk_len_bytes) as usize;

                        let max_len = (sectors as usize) * 4096;
                        if chunk_len == 0 || chunk_len > max_len || chunk_len > 2_000_000 {
                            continue;
                        }

                        let mut comp_scheme = [0u8; 1];
                        if file.read_exact(&mut comp_scheme).is_err() {
                            continue;
                        }

                        if comp_scheme[0] != 2 {
                            continue;
                        }

                        let mut comp_data = vec![0u8; chunk_len.saturating_sub(1)];
                        if file.read_exact(&mut comp_data).is_err() {
                            continue;
                        }

                        let mut decoder = ZlibDecoder::new(&comp_data[..]);
                        let mut decomp_data = Vec::new();
                        if decoder.read_to_end(&mut decomp_data).is_err() {
                            continue;
                        }

                        if let Ok(nbt) = fastnbt::from_bytes::<HashMap<String, NbtValue>>(&decomp_data) {
                            let mut chunk_colors = [[None; 16]; 16];
                            let mut filled_count = 0;

                            if let Some(NbtValue::List(sections)) = nbt.get("sections") {
                                for sec in sections.iter().rev() {
                                    if filled_count >= 256 {
                                        break;
                                    }

                                    if let NbtValue::Compound(sec_map) = sec {
                                        if let Some(NbtValue::Compound(block_states)) = sec_map.get("block_states") {
                                            let mut palette_names = Vec::new();
                                            if let Some(NbtValue::List(palette)) = block_states.get("palette") {
                                                for item in palette {
                                                    if let NbtValue::Compound(b_data) = item {
                                                        if let Some(NbtValue::String(name)) = b_data.get("Name") {
                                                            palette_names.push(name.as_str());
                                                        } else {
                                                            palette_names.push("minecraft:air");
                                                        }
                                                    }
                                                }
                                            }

                                            if palette_names.is_empty() || palette_names.iter().all(|n| self.ignored_blocks.contains(n)) {
                                                continue;
                                            }

                                            let packed_data = match block_states.get("data") {
                                                Some(NbtValue::LongArray(arr)) => Some(arr),
                                                _ => None,
                                            };

                                            let bits_per_block = std::cmp::max(4, 64 - (palette_names.len() as u64).saturating_sub(1).leading_zeros() as usize);
                                            let blocks_per_long = 64 / bits_per_block;
                                            let mask = (1u64 << bits_per_block) - 1;

                                            for bx in 0..16 {
                                                for bz in 0..16 {
                                                    if chunk_colors[bx][bz].is_some() {
                                                        continue;
                                                    }

                                                    for by in (0..16).rev() {
                                                        let pal_idx = if let Some(longs) = packed_data {
                                                            let block_index = (by * 16 + bz) * 16 + bx;
                                                            let long_idx = block_index / blocks_per_long;
                                                            let bit_offset = (block_index % blocks_per_long) * bits_per_block;
                                                            if long_idx < longs.len() {
                                                                ((longs[long_idx] as u64 >> bit_offset) & mask) as usize
                                                            } else {
                                                                0
                                                            }
                                                        } else {
                                                            0
                                                        };

                                                        if pal_idx < palette_names.len() {
                                                            let block_name = palette_names[pal_idx];
                                                            if !self.ignored_blocks.contains(&block_name) {
                                                                let col = self.color_map.get(block_name)
                                                                    .copied()
                                                                    .unwrap_or_else(|| self.guess_color(block_name));
                                                                chunk_colors[bx][bz] = Some(col);
                                                                filled_count += 1;
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            for bx in 0..16 {
                                for bz in 0..16 {
                                    let col = chunk_colors[bx][bz].unwrap_or([89, 166, 68, 255]);
                                    let px = ((rx + radius) * 512 + (cx as i32) * 16 + (bx as i32)) as u32;
                                    let pz = ((rz + radius) * 512 + (cz as i32) * 16 + (bz as i32)) as u32;
                                    if px < map_dim && pz < map_dim {
                                        img.put_pixel(px, pz, Rgba(col));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let resized = image::imageops::resize(&img, 1024, 1024, FilterType::Nearest);
        let mut buf = Cursor::new(Vec::new());
        if resized.write_to(&mut buf, ImageFormat::Png).is_ok() {
            format!("data:image/png;base64,{}", BASE64.encode(buf.into_inner()))
        } else {
            String::new()
        }
    }
}