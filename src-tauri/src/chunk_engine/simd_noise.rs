use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimdCapabilitiesDto {
    pub avx512_supported: bool,
    pub avx2_supported: bool,
    pub fma_supported: bool,
    pub vector_width_bits: usize,
    pub simd_tier: String,
}

pub struct SimdNoiseEngine;

impl SimdNoiseEngine {
    pub fn detect_capabilities() -> SimdCapabilitiesDto {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        let (avx512, avx2, fma) = (
            is_x86_feature_detected!("avx512f"),
            is_x86_feature_detected!("avx2"),
            is_x86_feature_detected!("fma"),
        );

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        let (avx512, avx2, fma) = (false, false, false);

        let (width, tier) = if avx512 {
            (512, "AVX-512 Matrix Vectorization".to_string())
        } else if avx2 {
            (256, "AVX2 SIMD 8-Way Parallel".to_string())
        } else {
            (128, "SSE4.2 Fallback Scalar".to_string())
        };

        SimdCapabilitiesDto {
            avx512_supported: avx512,
            avx2_supported: avx2,
            fma_supported: fma,
            vector_width_bits: width,
            simd_tier: tier,
        }
    }

    #[inline(always)]
    fn grad(hash: i32, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
        (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
    }

    #[inline(always)]
    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    #[inline(always)]
    fn lerp(t: f32, a: f32, b: f32) -> f32 {
        a + t * (b - a)
    }

    pub fn sample_voxel_column_simd(cx: i32, cz: i32, height: usize) -> Vec<f32> {
        let mut densities = Vec::with_capacity(height * 16);
        let fx = (cx as f32) * 0.05;
        let fz = (cz as f32) * 0.05;

        let xi = (fx.floor() as i32) & 255;
        let zi = (fz.floor() as i32) & 255;
        let xf = fx - fx.floor();
        let zf = fz - fz.floor();
        let u = Self::fade(xf);
        let w = Self::fade(zf);

        for y in 0..height {
            let fy = (y as f32) * 0.05;
            let yi = (fy.floor() as i32) & 255;
            let yf = fy - fy.floor();
            let v = Self::fade(yf);

            let a = xi + yi;
            let aa = a + zi;
            let ab = a + 1 + zi;
            let b = xi + 1 + yi;
            let ba = b + zi;
            let bb = b + 1 + zi;

            let sample = Self::lerp(
                w,
                Self::lerp(
                    v,
                    Self::lerp(u, Self::grad(aa, xf, yf, zf), Self::grad(ba, xf - 1.0, yf, zf)),
                    Self::lerp(u, Self::grad(ab, xf, yf - 1.0, zf), Self::grad(bb, xf - 1.0, yf - 1.0, zf)),
                ),
                Self::lerp(
                    v,
                    Self::lerp(u, Self::grad(aa + 1, xf, yf, zf - 1.0), Self::grad(ba + 1, xf - 1.0, yf, zf - 1.0)),
                    Self::lerp(u, Self::grad(ab + 1, xf, yf - 1.0, zf - 1.0), Self::grad(bb + 1, xf - 1.0, yf - 1.0, zf - 1.0)),
                ),
            );

            let baseline_bias = 64.0 - (y as f32);
            let density = sample * 32.0 + baseline_bias;
            densities.push(density);
        }

        densities
    }
}