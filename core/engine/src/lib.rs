pub use bytemuck::Zeroable;
pub use frenderer::{
    input::{Input, Key},
    wgpu, BitFont, Frenderer, GPUCamera as Camera, SheetRegion, Transform,
};
use structs::{ArrowREvent, REvent};
pub use std::fs::File;
pub use std::io::{self, BufRead};
pub use std::path::Path;
pub use std::collections::VecDeque;
pub trait Game: Sized + 'static {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine);
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
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
}

pub struct LevelState {
    pub loaded: bool,
    pub r_queue: VecDeque<Box<dyn REvent>>,
}

impl Engine {
    pub fn new(builder: winit::window::WindowBuilder) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window = builder.build(&event_loop).unwrap();
        let level_state = empty_level_state();
        let renderer = frenderer::with_default_runtime(&window);
        let input = Input::default();
        Self {
            renderer,
            input,
            level_state,
            window,
            event_loop: Some(event_loop),
        }
    }
    pub fn run<G: Game>(mut self) {
        let mut game = G::new(&mut self);
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
                            acc -= DT;
                            game.update(&mut self);
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

pub fn load_stage(engine: &mut Engine) {
    let mut r_queue: VecDeque<Box<dyn REvent>> = VecDeque::new();
    if let Ok(lines) = read_lines("content/levels/test_stage.rchart") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                // Read individual line parts
                let string_parts = ip.split(", ");
                let params = string_parts.collect::<Vec<&str>>();

                if params.len() > 0 {
                    match params[0] {
                        "Song" => {
    
                        }
                        "BPM" => {
                            // TODO Set BPM
                        }
                        "Arrow" => {
                            let mut e = ArrowREvent::zeroed();
                            e.load_event_from_string(params);
                            r_queue.push_back(Box::new(e));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    engine.level_state = LevelState {
        loaded: true,
        r_queue: r_queue,
    };
}

fn empty_level_state() -> LevelState {
    return LevelState { loaded: false, r_queue: VecDeque::new() };
}
