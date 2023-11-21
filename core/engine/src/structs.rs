pub use glam::*;
use bytemuck::Zeroable;

pub struct Arrow {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub spin: f32,
<<<<<<< HEAD
    pub target_time: usize,
=======
>>>>>>> arrow-work
    // 0 = left, 1 = right, 2 = up, 3 = down
    pub arrow_dir: usize,
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

pub trait REvent {
    fn spawn_event(&self);
    fn get_start_time(&self) -> usize;
    fn load_event_from_string(&mut self, file_line: Vec<&str>);
}

#[derive(Zeroable)]
pub struct ArrowREvent {
    pub start_time: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub spin: f32,
    pub target_time: usize,
}

impl REvent for ArrowREvent {
    fn spawn_event(&self) {
        // TODO
    }

    fn get_start_time(&self) -> usize {
        return self.start_time;
    }

    fn load_event_from_string(&mut self, file_line: Vec<&str>) {
        let mut start_pos = 0.0;
        let mut target_pos = 0.0;
        let mut start_time: usize = 0;
        let mut target_time: usize = 0;
        let mut start_rot = 0.0;
        let mut end_rot = 0.0;
        // TODO
        for i in 1..file_line.len()
        {
            match file_line.get(i) {
                Some(text) => {
                    match i {
                        1 => { start_pos = text.parse().unwrap(); }
                        2 => { target_pos = text.parse().unwrap(); }
                        3 => { start_time = text.parse().unwrap(); }
                        4 => { target_time = text.parse().unwrap(); }
                        5 => { start_rot = text.parse().unwrap(); }
                        6 => { end_rot = text.parse().unwrap(); }
                        _ => {
                            println!("Unimplemented id {} reached.", i);
                        }
                    }
                    dbg!(text);
                }
                _ => {println!("Error, invalid input string at point {}.", i)}
            }
        }
        self.start_time = start_time;
        //self.pos = start_pos;
        //self.vel = (target_pos - start_pos) / (target_time - start_time);
        self.target_time = target_time;
    }
}
