use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use rdev::{listen, Event, EventType, Key};

pub struct OverlayService;

impl OverlayService {
    pub fn start_hotkey_listener<F>(stop_signal: Arc<AtomicBool>, on_toggle: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let toggle_cb = Arc::new(on_toggle);
        let shift_pressed = Arc::new(AtomicBool::new(false));
        let last_trigger_millis = Arc::new(AtomicU64::new(0));

        std::thread::spawn(move || {
            let cb = Arc::clone(&toggle_cb);
            let shift_state = Arc::clone(&shift_pressed);
            let trigger_timer = Arc::clone(&last_trigger_millis);

            let _ = std::panic::catch_unwind(AssertUnwindSafe(move || {
                let _ = listen(move |event: Event| {
                    if stop_signal.load(Ordering::Relaxed) {
                        return;
                    }

                    match event.event_type {
                        EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => {
                            shift_state.store(true, Ordering::Relaxed);
                        }
                        EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => {
                            shift_state.store(false, Ordering::Relaxed);
                        }
                        EventType::KeyPress(Key::Tab) => {
                            if shift_state.load(Ordering::Relaxed) {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map(|d| d.as_millis() as u64)
                                    .unwrap_or(0);

                                let previous = trigger_timer.load(Ordering::Relaxed);
                                if now.saturating_sub(previous) >= 350 {
                                    trigger_timer.store(now, Ordering::Relaxed);
                                    cb();
                                }
                            }
                        }
                        _ => {}
                    }
                });
            }));
        });
    }
}