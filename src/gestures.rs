use std::collections::HashMap;

use winit::{dpi::PhysicalPosition, event::Touch, event::TouchPhase};

struct FingerState {
    current: PhysicalPosition<f64>,
    motion_vector: PhysicalPosition<f64>
}

impl FingerState {
    pub fn new(current: PhysicalPosition<f64>) -> Self {
        Self {
            current,
            motion_vector: PhysicalPosition { x: 0.0, y: 0.0 }
        }
    }
    pub fn move_to(&mut self, new: PhysicalPosition<f64>) {
        self.motion_vector = PhysicalPosition { x: new.x - self.current.x, y: new.y - self.current.y };
        self.current = new;
    }
    pub fn motion_vector(&self) -> &PhysicalPosition<f64> {
        &self.motion_vector
    }
}
pub struct GestureState {
    finger_state: HashMap<u64, FingerState>
}

impl GestureState {
    pub fn new() -> Self {
        Self {
            finger_state: HashMap::new()
        }
    }
    pub fn on_touch_event(&mut self, event: &Touch) {
        match event.phase {
            TouchPhase::Started => {
                self.finger_state.insert(event.id, FingerState::new(event.location));
            },
            TouchPhase::Moved => {
                self.finger_state.get_mut(&event.id).map(|finger| finger.move_to(event.location));
            },
            TouchPhase::Ended => {
                self.finger_state.remove(&event.id);
            }
            TouchPhase::Cancelled => {
                self.finger_state.remove(&event.id);
            },
        }
    }
    pub fn drag(&mut self) -> (f32, f32) {
        if self.finger_state.is_empty() {
            return (0.0, 0.0);
        }

        let mut avg = PhysicalPosition::new(0.0, 0.0);
        for (_, finger) in self.finger_state.iter_mut() {
            avg.x += finger.motion_vector.x;
            avg.y += finger.motion_vector.y;

            // RESET AFTER CONSUME
            finger.motion_vector = PhysicalPosition::new(0.0, 0.0);
        }

        avg.x /= self.finger_state.len() as f64;
        avg.y /= self.finger_state.len() as f64;

        (avg.x as f32, avg.y as f32)
    }
}