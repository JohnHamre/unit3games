pub use glam::*;

pub struct Arrow {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub target: Vec2,
    pub target_frame: usize,
}

impl Arrow {
    // Function used to get the speed of an arrow in motion.
    pub fn speed(&self) -> f32 {
        return (self.vel.x.powf(2.0) + self.vel.y.powf(2.0)).sqrt();
    }

    // Called each simulation frame for each arrow.
    pub fn update_arrow() {
        // Move the arrow

        // Detect tap?

        // Notify and despawn if missed.
    }
}

pub struct Target {
    pub pos: Vec2,
}