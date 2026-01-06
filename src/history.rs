use crate::graph::BlueprintGraph;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UndoStack {
    pub history: Vec<BlueprintGraph>,
    pub current_index: usize,
    pub max_records: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            current_index: 0,
            max_records: 1000,
        }
    }
}

impl UndoStack {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn push(&mut self, graph: &BlueprintGraph) {
        // If we are not at the end, truncate future
        if self.current_index + 1 < self.history.len() {
            self.history.truncate(self.current_index + 1);
        }
        
        self.history.push(graph.clone());
        self.current_index = self.history.len() - 1;
        
        if self.history.len() > self.max_records {
            self.history.remove(0);
            self.current_index = self.current_index.saturating_sub(1);
        }
    }
    
    pub fn undo(&mut self) -> Option<BlueprintGraph> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.history.get(self.current_index).cloned()
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<BlueprintGraph> {
        if self.current_index + 1 < self.history.len() {
            self.current_index += 1;
            self.history.get(self.current_index).cloned()
        } else {
            None
        }
    }
}
