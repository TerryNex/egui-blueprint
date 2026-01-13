use crossbeam_channel::{Receiver, unbounded};
use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use enigo::{Enigo, Settings, Mouse, Coordinate};

pub mod mapper;

pub struct RecordedAction {
    pub event: Event,
    pub cursor_position: (f64, f64),
}

/// Get current cursor position using enigo
fn get_cursor_position() -> (f64, f64) {
    if let Ok(enigo) = Enigo::new(&Settings::default()) {
        if let Ok((x, y)) = enigo.location() {
            return (x as f64, y as f64);
        }
    }
    (0.0, 0.0)
}

pub struct Recorder {
    pub rx: Receiver<RecordedAction>,
    is_recording: Arc<AtomicBool>,
    recording_start_time_ms: Arc<AtomicU64>,
    record_mouse_move: Arc<AtomicBool>,
}

impl Recorder {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let is_recording = Arc::new(AtomicBool::new(false));
        let is_recording_clone = is_recording.clone();
        let recording_start_time_ms = Arc::new(AtomicU64::new(0));
        let recording_start_time_ms_clone = recording_start_time_ms.clone();
        let record_mouse_move = Arc::new(AtomicBool::new(false));
        let record_mouse_move_clone = record_mouse_move.clone();

        thread::spawn(move || {
            let mut last_recorded_position = (0.0f64, 0.0f64);
            let mut last_recorded_time = Instant::now();

            if let Err(error) = listen(move |event| {
                // ESC Key: Stop Recording
                if let EventType::KeyPress(Key::Escape) = event.event_type {
                    if is_recording_clone.load(Ordering::Relaxed) {
                        is_recording_clone.store(false, Ordering::Relaxed);
                        return;
                    }
                }

                if is_recording_clone.load(Ordering::Relaxed) {
                    // Cooldown: Ignore events within 500ms of starting recording
                    let start_ms = recording_start_time_ms_clone.load(Ordering::Relaxed);
                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    
                    if now_ms < start_ms + 500 {
                        return;
                    }

                    // Get REAL current cursor position for all events
                    let current_pos = get_cursor_position();

                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            if !record_mouse_move_clone.load(Ordering::Relaxed) {
                                return;
                            }
                            
                            let now = Instant::now();
                            let dx = (x - last_recorded_position.0).abs();
                            let dy = (y - last_recorded_position.1).abs();
                            
                            if (dx > 10.0 || dy > 10.0) && now.duration_since(last_recorded_time) > Duration::from_millis(250) {
                                let _ = tx.send(RecordedAction {
                                    event: event.clone(),
                                    cursor_position: current_pos,
                                });
                                last_recorded_position = (x, y);
                                last_recorded_time = now;
                            }
                        }
                        EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                            // For mouse button events, ALWAYS use real-time position from enigo
                            let _ = tx.send(RecordedAction {
                                event,
                                cursor_position: current_pos,
                            });
                            last_recorded_position = current_pos;
                            last_recorded_time = Instant::now();
                        }
                        EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                            // For keyboard events
                            let _ = tx.send(RecordedAction {
                                event,
                                cursor_position: current_pos,
                            });
                        }
                        _ => {}
                    }
                }
            }) {
                println!("Error in Recorder listener: {:?}", error);
            }
        });

        Self { rx, is_recording, recording_start_time_ms, record_mouse_move }
    }

    pub fn start(&self) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.recording_start_time_ms.store(now_ms, Ordering::Relaxed);
        self.is_recording.store(true, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::Relaxed);
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
    
    pub fn set_record_mouse_move(&self, enabled: bool) {
        self.record_mouse_move.store(enabled, Ordering::Relaxed);
    }
    
    pub fn is_record_mouse_move(&self) -> bool {
        self.record_mouse_move.load(Ordering::Relaxed)
    }
}
