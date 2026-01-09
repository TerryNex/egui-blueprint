use crate::graph::Node;
use crossbeam_channel::{Receiver, unbounded};
use rdev::{Event, EventType, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub mod mapper;

pub struct RecordedAction {
    pub event: Event,
    pub cursor_position: (f64, f64),
}

pub struct Recorder {
    pub rx: Receiver<RecordedAction>,
    is_recording: Arc<AtomicBool>,
}

impl Recorder {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let is_recording = Arc::new(AtomicBool::new(false));
        let is_recording_clone = is_recording.clone();

        thread::spawn(move || {
            let mut last_position = (0.0, 0.0);

            if let Err(error) = listen(move |event| {
                // Update position if it's a move event
                if let EventType::MouseMove { x, y } = event.event_type {
                    last_position = (x, y);
                }

                if is_recording_clone.load(Ordering::Relaxed) {
                    match event.event_type {
                        EventType::MouseMove { .. } => {
                            // Ignore Move events for recording logic (too verbose)
                        }
                        _ => {
                            let _ = tx.send(RecordedAction {
                                event,
                                cursor_position: last_position,
                            });
                        }
                    }
                }
            }) {
                println!("Error in Recorder listener: {:?}", error);
            }
        });

        Self { rx, is_recording }
    }

    pub fn start(&self) {
        self.is_recording.store(true, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.is_recording.store(false, Ordering::Relaxed);
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::Relaxed)
    }
}
