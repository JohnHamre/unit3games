pub use bytemuck::Zeroable;
pub use frenderer::{
    input::{Input, Key},
    wgpu, BitFont, Frenderer, GPUCamera as Camera, SheetRegion, Transform,
};
pub use glam::*;
use structs::{REvent, Arrow};
pub use std::fs::File;
pub use std::io::{self, BufRead};
pub use std::path::Path;
pub use std::collections::VecDeque;
use kira::{
	manager::{
		AudioManager, AudioManagerSettings,
		backend::DefaultBackend,
	},
	sound::static_sound::{StaticSoundData, StaticSoundSettings}
};

use crate::structs::Spawnable;
pub trait Game: Sized + 'static {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine, frame_events: &mut VecDeque<REvent>);
    fn render(&mut self, engine: &mut Engine);
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub struct Engine {
    pub renderer: Frenderer,
    pub input: Input,
    pub level_state: LevelState,
    pub song_name: String,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
}

pub struct LevelState {
    pub loaded: bool,
    pub queue_timer: i32,
    pub r_queue: VecDeque<REvent>,
}

impl Engine {
    pub fn new(builder: winit::window::WindowBuilder) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = builder.build(&event_loop).unwrap();
        let level_state = empty_level_state();
        let song_name: String = "".to_string();
        let renderer = frenderer::with_default_runtime(&window);
        let input = Input::default();
        Self {
            renderer,
            input,
            level_state,
            song_name,
            window,
            event_loop: Some(event_loop),
        }
    }
    pub fn run<G: Game>(mut self) {
        let mut game = G::new(&mut self);
        let mut frame_events: VecDeque<REvent> = VecDeque::new();
        /*
        -------------------------------------
            Stage Loading + Song Init        
        -------------------------------------
         */
        load_stage(&mut self, "content/levels/test_stage.rchart");
        let mut song_player = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();
        const DT: f32 = 1.0 / 240.0;
        const DT_FUDGE_AMOUNT: f32 = 0.000002;
        const DT_MAX: f32 = DT * 5.0;
        const TIME_SNAPS: [f32; 5] = [15.0, 30.0, 60.0, 120.0, 144.0];
        let mut acc = 0.0;
        let mut now = std::time::Instant::now();
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, _, control_flow| {
                use winit::event::{Event, WindowEvent};
                control_flow.set_poll();
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    Event::MainEventsCleared => {
                        // compute elapsed time since last frame
                        let mut elapsed = now.elapsed().as_secs_f32();
                        // println!("{elapsed}");
                        // snap time to nearby vsync framerate
                        TIME_SNAPS.iter().for_each(|s| {
                            if (elapsed - 1.0 / s).abs() < DT_FUDGE_AMOUNT {
                                elapsed = 1.0 / s;
                            }
                        });
                        // Death spiral prevention
                        if elapsed > DT_MAX {
                            acc = 0.0;
                            elapsed = DT;
                        }
                        acc += elapsed;
                        now = std::time::Instant::now();
                        // While we have time to spend
                        while acc >= DT {
                            // simulate a frame

                            self.level_state.queue_timer += 1;

                            // Start the song
                            if self.level_state.queue_timer == 0 {
                                let sound_data =
                                    StaticSoundData::from_file(&self.song_name, 
                                    StaticSoundSettings::default()).unwrap();

                                let _ = song_player.play(sound_data.clone());
                            }

                            // Syntax, wow :o
                            // self.level_state.r_queue.get(0).map(|event| event.get_start_time() <= self.level_state.queue_timer).unwrap_or(false)
                            loop
                            {
                                match self.level_state.r_queue.get(0) {
                                    Some(REvent::ArrowEvent(a)) => {
                                        if a.get_start_time() <= self.level_state.queue_timer {
                                            frame_events.push_back(self.level_state.r_queue.pop_front().unwrap());
                                            println!("{}", self.level_state.queue_timer);
                                        }
                                        else {
                                            break;
                                        }
                                    }
                                    None => {break;}
                                    _ => {                                            
                                        frame_events.push_back(self.level_state.r_queue.pop_front().unwrap());
                                        println!("{}", self.level_state.queue_timer);
                                    }
                                }
                            }
                            acc -= DT;
                            game.update(&mut self, &mut frame_events);
                            frame_events.clear();
                            self.input.next_frame();
                        }
                        game.render(&mut self);
                        // Render prep
                        //self.renderer.sprites.set_camera_all(&frend.gpu, camera);
                        // update sprite positions and sheet regions
                        // ok now render.
                        // We could just call frend.render().
                        self.renderer.render();
                        self.window.request_redraw();
                    }
                    event => {
                        if self.renderer.process_window_event(&event) {
                            self.window.request_redraw();
                        }
                        self.input.process_input_event(&event);
                    }
                }
            });
    }
}
pub mod geom;
pub mod structs;

pub fn load_stage(engine: &mut Engine, filepath: &str) {
    let mut r_queue: VecDeque<REvent> = VecDeque::new();
    if let Ok(lines) = read_lines(filepath) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                // Read individual line parts
                let string_parts = ip.split(", ");
                let params = string_parts.collect::<Vec<&str>>();

                if params.len() > 0 {
                    match params[0] {
                        "Song" => {
                            engine.song_name = params[1].to_string();
                        }
                        "BPM" => {
                            // TODO Set BPM
                        }
                        "Arrow" => {
                            let mut e = Arrow::zeroed();
                            e.load_event_from_string(params);
                            r_queue.push_back(REvent::ArrowEvent(e));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    engine.level_state = LevelState {
        loaded: true,
        queue_timer: - (240),
        r_queue: r_queue,
    };
}

fn empty_level_state() -> LevelState {
    return LevelState { loaded: false, queue_timer: 0, r_queue: VecDeque::new() };
}
