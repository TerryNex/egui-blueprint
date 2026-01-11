use crossbeam_channel::{Receiver, unbounded};
use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};

pub mod mapper;

pub struct RecordedAction {
    pub event: Event,
    pub cursor_position: (f64, f64),
}

pub struct Recorder {
    pub rx: Receiver<RecordedAction>,
    is_recording: Arc<AtomicBool>,
    recording_start_time_ms: Arc<AtomicU64>, // Unix timestamp in ms when recording started
}

impl Recorder {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let is_recording = Arc::new(AtomicBool::new(false));
        let is_recording_clone = is_recording.clone();
        let recording_start_time_ms = Arc::new(AtomicU64::new(0));
        let recording_start_time_ms_clone = recording_start_time_ms.clone();

        thread::spawn(move || {
            let mut last_position = (0.0, 0.0);
            let mut last_recorded_position = (0.0f64, 0.0f64);
            let mut last_recorded_time = Instant::now();

            if let Err(error) = listen(move |event| {
                // Update position if it's a move event
                if let EventType::MouseMove { x, y } = event.event_type {
                    last_position = (x, y);
                }

                // ESC Key: Stop Recording
                if let EventType::KeyPress(Key::Escape) = event.event_type {
                    if is_recording_clone.load(Ordering::Relaxed) {
                        is_recording_clone.store(false, Ordering::Relaxed);
                        return; // Don't record the ESC key itself
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
                        // Within cooldown, ignore this event
                        return;
                    }

                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            // Record moves if significant distance AND enough time passed
                            let now = Instant::now();
                            let dx = (x - last_recorded_position.0).abs();
                            let dy = (y - last_recorded_position.1).abs();
                            
                            // Throttle: >10px AND >250ms interval
                            if (dx > 10.0 || dy > 10.0) && now.duration_since(last_recorded_time) > Duration::from_millis(250) {
                                let _ = tx.send(RecordedAction {
                                    event: event.clone(),
                                    cursor_position: last_position,
                                });
                                last_recorded_position = (x, y);
                                last_recorded_time = now;
                            }
                        }
                        _ => {
                            let _ = tx.send(RecordedAction {
                                event,
                                cursor_position: last_position,
                            });
                            // Reset timer/pos on clicks
                            last_recorded_position = last_position;
                            last_recorded_time = Instant::now();
                        }
                    }
                }
            }) {
                println!("Error in Recorder listener: {:?}", error);
            }
        });

        Self { rx, is_recording, recording_start_time_ms }
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
}
