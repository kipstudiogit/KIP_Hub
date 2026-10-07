use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckerboardTopologyDto {
    pub current_phase: u8,
    pub phase_name: String,
    pub queued_chunks: usize,
    pub resolved_cascades_count: usize,
    pub thread_concurrency_tier: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
}

pub struct CheckerboardScheduler {
    phase_0: Arc<Mutex<VecDeque<ChunkCoordinate>>>,
    phase_1: Arc<Mutex<VecDeque<ChunkCoordinate>>>,
    phase_2: Arc<Mutex<VecDeque<ChunkCoordinate>>>,
    phase_3: Arc<Mutex<VecDeque<ChunkCoordinate>>>,
    current_phase: Arc<AtomicUsize>,
    resolved_cascades: Arc<AtomicUsize>,
}

impl CheckerboardScheduler {
    pub fn new() -> Self {
        Self {
            phase_0: Arc::new(Mutex::new(VecDeque::new())),
            phase_1: Arc::new(Mutex::new(VecDeque::new())),
            phase_2: Arc::new(Mutex::new(VecDeque::new())),
            phase_3: Arc::new(Mutex::new(VecDeque::new())),
            current_phase: Arc::new(AtomicUsize::new(0)),
            resolved_cascades: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn reset(&self) {
        self.phase_0.lock().clear();
        self.phase_1.lock().clear();
        self.phase_2.lock().clear();
        self.phase_3.lock().clear();
        self.current_phase.store(0, Ordering::SeqCst);
        self.resolved_cascades.store(0, Ordering::SeqCst);
    }

    pub fn schedule_coordinate(&self, coord: ChunkCoordinate) {
        let x_even = coord.x.rem_euclid(2) == 0;
        let z_even = coord.z.rem_euclid(2) == 0;

        match (x_even, z_even) {
            (true, true) => self.phase_0.lock().push_back(coord),
            (false, false) => self.phase_1.lock().push_back(coord),
            (true, false) => self.phase_2.lock().push_back(coord),
            (false, true) => self.phase_3.lock().push_back(coord),
        }

        self.resolved_cascades.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pop_batch_for_phase(&self, max_batch: usize) -> (u8, Vec<ChunkCoordinate>) {
        let phase = (self.current_phase.load(Ordering::Relaxed) % 4) as u8;
        let mut results = Vec::with_capacity(max_batch);

        match phase {
            0 => {
                let mut guard = self.phase_0.lock();
                for _ in 0..max_batch {
                    if let Some(c) = guard.pop_front() {
                        results.push(c);
                    } else {
                        break;
                    }
                }
            }
            1 => {
                let mut guard = self.phase_1.lock();
                for _ in 0..max_batch {
                    if let Some(c) = guard.pop_front() {
                        results.push(c);
                    } else {
                        break;
                    }
                }
            }
            2 => {
                let mut guard = self.phase_2.lock();
                for _ in 0..max_batch {
                    if let Some(c) = guard.pop_front() {
                        results.push(c);
                    } else {
                        break;
                    }
                }
            }
            3 => {
                let mut guard = self.phase_3.lock();
                for _ in 0..max_batch {
                    if let Some(c) = guard.pop_front() {
                        results.push(c);
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }

        self.current_phase.fetch_add(1, Ordering::Relaxed);
        (phase, results)
    }

    pub fn get_topology_metrics(&self) -> CheckerboardTopologyDto {
        let phase = (self.current_phase.load(Ordering::Relaxed) % 4) as u8;
        let total_queued = self.phase_0.lock().len()
            + self.phase_1.lock().len()
            + self.phase_2.lock().len()
            + self.phase_3.lock().len();

        let phase_name = match phase {
            0 => "Phase 0 [Even, Even] - Primary Slabs".to_string(),
            1 => "Phase 1 [Odd, Odd] - Inverse Islands".to_string(),
            2 => "Phase 2 [Even, Odd] - Cross Bridges".to_string(),
            _ => "Phase 3 [Odd, Even] - Boundary Stitch".to_string(),
        };

        CheckerboardTopologyDto {
            current_phase: phase,
            phase_name,
            queued_chunks: total_queued,
            resolved_cascades_count: self.resolved_cascades.load(Ordering::Relaxed),
            thread_concurrency_tier: "Zero-Lock 2D Cellular Automaton".to_string(),
        }
    }
}