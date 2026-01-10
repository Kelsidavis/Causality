// Undo/Redo history system for the editor
//
// Uses a snapshot-based approach: stores serialized scene states in stacks.
// Simple and robust - works for all operations automatically.

use engine_scene::{scene::Scene, scene_data::SerializedScene};

/// Manages undo/redo history using scene snapshots
pub struct UndoHistory {
    /// Stack of previous states (for undo)
    undo_stack: Vec<SerializedScene>,
    /// Stack of future states (for redo, populated after undo)
    redo_stack: Vec<SerializedScene>,
    /// Maximum number of snapshots to keep
    max_history: usize,
}

impl UndoHistory {
    /// Create a new undo history with specified maximum size
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Save the current scene state before making a change.
    /// Call this BEFORE modifying the scene.
    pub fn push_state(&mut self, scene: &Scene) {
        let snapshot = scene.to_serialized();
        self.undo_stack.push(snapshot);

        // New action clears redo stack
        self.redo_stack.clear();

        // Limit history size to prevent unbounded memory growth
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the last change: restore previous state, save current to redo stack.
    /// Returns true if undo was performed.
    pub fn undo(&mut self, scene: &mut Scene) -> bool {
        if let Some(prev_state) = self.undo_stack.pop() {
            // Save current state to redo stack before restoring
            self.redo_stack.push(scene.to_serialized());
            // Restore previous state
            *scene = Scene::from_serialized(prev_state);
            true
        } else {
            false
        }
    }

    /// Redo the last undone change: restore next state, save current to undo stack.
    /// Returns true if redo was performed.
    pub fn redo(&mut self, scene: &mut Scene) -> bool {
        if let Some(next_state) = self.redo_stack.pop() {
            // Save current state to undo stack before restoring
            self.undo_stack.push(scene.to_serialized());
            // Restore next state
            *scene = Scene::from_serialized(next_state);
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo steps available
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo steps available
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history (e.g., when loading a new scene)
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new(50)
    }
}
