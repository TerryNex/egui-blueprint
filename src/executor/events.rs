use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    Log(String),
    NodeActive(Uuid),
    NodeInactive(Uuid), // Optional, for finer control if needed later
    Finished,
}
