pub use glam::*;
use bytemuck::Zeroable;

pub struct Arrow {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rot: f32,
    pub spin: f32,
    pub target_time: i32,
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
    fn get_start_time(&self) -> i32;
    fn load_event_from_string(&mut self, file_line: Vec<&str>);
}

#[derive(Zeroable)]
pub struct ArrowREvent {
    pub start_time: i32,
    pub pos: Vec2,
    pub vel: Vec2,
    pub arrow_dir: usize,
    pub rot: f32,
    pub spin: f32,
    pub target_time: i32,
}

impl REvent for ArrowREvent {
    fn spawn_event(&self) {
        // handled game side, so does nothing
        //game.spawn_arrow(self.pos, self.vel, self.rot, self.spin, self.arrow_dir, self.target_time);
    }

    fn get_start_time(&self) -> i32 {
        return self.start_time;
    }

    fn load_event_from_string(&mut self, file_line: Vec<&str>) {
        let mut start_pos = Vec2::zeroed();
        let mut target_pos = Vec2::zeroed();
        let mut arrow_dir: usize = 0;
        let mut start_time: i32 = 0;
        let mut target_time: i32 = 0;
        let mut start_rot = 0.0;
        let mut end_rot = 0.0;
        // TODO
        for i in 1..file_line.len()
        {
            match file_line.get(i) {
                Some(text) => {
                    match i {
                        1 => { start_pos.x = text.parse().unwrap(); }
                        2 => { start_pos.y = text.parse().unwrap(); }
                        3 => { target_pos.x = text.parse().unwrap(); }
                        4 => { target_pos.y = text.parse().unwrap(); }
                        5 => { arrow_dir = text.parse().unwrap(); }
                        6 => { start_time = text.parse().unwrap(); }
                        7 => { target_time = text.parse().unwrap(); }
                        8 => { start_rot = text.parse().unwrap(); }
                        9 => { end_rot = text.parse().unwrap(); }
                        _ => {
                            //println!("Unimplemented id {} reached.", i);
                        }
                    }
                }
                _ => {/*println!("Error, invalid input string at point {}.", i)*/}
            }
        }
        self.start_time = start_time;
        self.pos = start_pos;
        self.vel = Vec2{
            x: (target_pos.x - start_pos.x) / (target_time - start_time) as f32,
            y: (target_pos.y - start_pos.y) / (target_time - start_time) as f32,
        };
        self.arrow_dir = arrow_dir;
        self.target_time = target_time;
        self.rot = start_rot;
        // TODO, this one is gonna take more math than I can be bothered to work out tn.
        self.spin = end_rot;
    }
}
