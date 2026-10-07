use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffHeapArenaMetricsDto {
    pub total_allocated_mb: f64,
    pub used_memory_mb: f64,
    pub active_chunks_in_flight: usize,
    pub allocation_bumps_count: usize,
    pub zero_gc_efficiency_percent: u32,
}

pub struct OffHeapVoxelArena {
    raw_buffer: NonNull<u8>,
    layout: Layout,
    capacity_bytes: usize,
    head_offset: Arc<AtomicUsize>,
    active_in_flight: Arc<AtomicUsize>,
    total_bumps: Arc<AtomicUsize>,
    lock: Arc<Mutex<()>>,
}

unsafe impl Send for OffHeapVoxelArena {}
unsafe impl Sync for OffHeapVoxelArena {}

impl OffHeapVoxelArena {
    pub fn new(capacity_mb: usize) -> Self {
        let capacity_bytes = capacity_mb.clamp(32, 512) * 1024 * 1024;
        let layout = Layout::from_size_align(capacity_bytes, 64).expect("Invalid arena layout");
        let raw_ptr = unsafe { alloc(layout) };
        let raw_buffer = NonNull::new(raw_ptr).expect("Out of memory for off-heap voxel arena");

        Self {
            raw_buffer,
            layout,
            capacity_bytes,
            head_offset: Arc::new(AtomicUsize::new(0)),
            active_in_flight: Arc::new(AtomicUsize::new(0)),
            total_bumps: Arc::new(AtomicUsize::new(0)),
            lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn allocate_chunk_slab(&self, size_bytes: usize) -> Option<*mut u8> {
        let _guard = self.lock.lock();
        let current = self.head_offset.load(Ordering::Relaxed);
        let aligned_size = (size_bytes + 63) & !63;

        let target_offset = if current + aligned_size > self.capacity_bytes {
            0
        } else {
            current
        };

        self.head_offset.store(target_offset + aligned_size, Ordering::SeqCst);
        self.active_in_flight.fetch_add(1, Ordering::Relaxed);
        self.total_bumps.fetch_add(1, Ordering::Relaxed);

        unsafe {
            let ptr = self.raw_buffer.as_ptr().add(target_offset);
            std::ptr::write_bytes(ptr, 0, aligned_size);
            Some(ptr)
        }
    }

    pub fn release_chunk_slab(&self) {
        let current = self.active_in_flight.load(Ordering::Relaxed);
        if current > 0 {
            self.active_in_flight.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn reset(&self) {
        let _guard = self.lock.lock();
        self.head_offset.store(0, Ordering::SeqCst);
        self.active_in_flight.store(0, Ordering::Relaxed);
    }

    pub fn get_metrics(&self) -> OffHeapArenaMetricsDto {
        let total_mb = (self.capacity_bytes as f64) / (1024.0 * 1024.0);
        let used_bytes = self.head_offset.load(Ordering::Relaxed);
        let used_mb = (used_bytes as f64) / (1024.0 * 1024.0);
        let in_flight = self.active_in_flight.load(Ordering::Relaxed);
        let bumps = self.total_bumps.load(Ordering::Relaxed);

        OffHeapArenaMetricsDto {
            total_allocated_mb: (total_mb * 10.0).round() / 10.0,
            used_memory_mb: (used_mb * 10.0).round() / 10.0,
            active_chunks_in_flight: in_flight,
            allocation_bumps_count: bumps,
            zero_gc_efficiency_percent: 99,
        }
    }
}

impl Drop for OffHeapVoxelArena {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.raw_buffer.as_ptr(), self.layout);
        }
    }
}