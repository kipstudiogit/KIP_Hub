use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use rdev::{listen, Event, EventType, Key};

pub struct OverlayService;

impl OverlayService {
    pub fn start_hotkey_listener<F>(stop_signal: Arc<AtomicBool>, on_toggle: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let toggle_cb = Arc::new(on_toggle);
        let shift_pressed = Arc::new(AtomicBool::new(false));

        std::thread::spawn(move || {
            let cb = Arc::clone(&toggle_cb);
            let shift_state = Arc::clone(&shift_pressed);

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
                            cb();
                        }
                    }
                    _ => {}
                }
            });
        });
    }
}