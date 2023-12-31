// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size
use std::fs;
use std::io::Write;
use engine as engine;
use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use engine::structs::*;
//use rand::Rng;
pub use engine::structs::Arrow;
pub use std::collections::VecDeque;
const W: f32 = 320.0;
const H: f32 = 240.0;
const SPRITE_MAX: usize = 16;

struct TargetData {
    pos: Vec2,
    arrow_dir: i32,
}

#[derive(Zeroable, Debug, Clone, Copy)]
struct TemporalMarker {
    measure: usize,
    beat: usize,
    sixteenth: usize,
}

#[derive(Zeroable, Debug, Copy, Clone)]
struct ArrowData {
    start_pos: Vec2,
    travel_time: i32,
    arrow_dir: usize,
    target_time: TemporalMarker,
}

impl ArrowData {
    fn location(&self) -> usize {
        return self.target_time.sixteenth + 4 * self.target_time.beat + 16 * self.target_time.measure;
    }
}

struct Game {
    camera: engine::Camera,
    targets: Vec<TargetData>,
    font: engine::BitFont,
    arrows0: Vec<ArrowData>,
    arrows1: Vec<ArrowData>,
    arrows2: Vec<ArrowData>,
    arrows3: Vec<ArrowData>,
    current_measure: usize,
    output_filepath: String,
    song: String,
    bpm: usize,
    num_beats: usize,
}

impl engine::Game for Game {
    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [W, H],
        };
        #[cfg(target_arch = "wasm32")]
        let sprite_img = {
            let img_bytes = include_bytes!("content/Sheet2.png");
            image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())
                .unwrap()
                .into_rgba8()
        };
        #[cfg(not(target_arch = "wasm32"))]
        let sprite_img = image::open("content/Sheet2.png").unwrap().into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-Sheet2.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            vec![Transform::zeroed(); SPRITE_MAX], //bg, three walls, guy, a few apples
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 220, 464, 12, 70, 10),
            10,
        );
        let paths = fs::read_dir("content/levels/").unwrap();
        for path in paths {
            let p1 = path.unwrap().path();
            let p_str = p1.to_str().unwrap();
            if p_str.len() > 7 {
                let last_seven = {
                    let split_pos = p_str.char_indices().nth_back(6).unwrap().0;
                    &p_str[split_pos..]
                };
                if ".rchart" == last_seven {
                    // Store to array instead of print.
                    //println!("Name: {}", p_str);
                }
            }
        }
        let mut targets = Vec::new();
        for i in 0..=3 {
            targets.push(TargetData{pos: Vec2::new(100.0 + 24.0 * i as f32, 200.0), arrow_dir: i});
        }
        Game {
            camera,
            targets,
            font,
            arrows0: Vec::new(),
            arrows1: Vec::new(),
            arrows2: Vec::new(),
            arrows3: Vec::new(),
            current_measure: 0,
            output_filepath: "bill_nye".to_string(),
            song: "william_overture.ogg".to_string(),
            bpm: 131,
            num_beats: 10,
        }
    }
    fn update(&mut self, engine: &mut Engine, _frame_events: &mut VecDeque<REvent>) {
        //let dir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);

        // Add note
        let mouse_pos = engine.input.mouse_pos();
        let mut col = 4;
        if mouse_pos.y >= 136.0 && mouse_pos.y <= 463.0 {
            if mouse_pos.x <= 60.0 {
                col = 0;
            }
            else if mouse_pos.x <= 120.0 {
                col = 1;
            }
            else if mouse_pos.x <= 180.0 {
                col = 2;
            }
            else if mouse_pos.x <= 240.0 {
                col = 3;
            }
        }
        if col < 4 {
            if engine.input.is_mouse_pressed(winit::event::MouseButton::Left) {
                spawn_arrow(self, col, mouse_pos.y)
            }
        }

        if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Down) {
            self.current_measure += 1;
        }
        if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Up) {
            if self.current_measure > 0 {
                self.current_measure -= 1;
            }
        }

        // Export
        if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Space) {
            export_stage(self);
        }
    }
    fn render(&mut self, engine: &mut Engine) {
        // set bg image
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
        trfs[0] = AABB {
            center: Vec2 {
                x: W / 2.0,
                y: H / 2.0,
            },
            size: Vec2 { x: W, y: H },
        }
        .into();
        uvs[0] = SheetRegion::new(0, 0, 0, 16, 880, 440);
        // TODO animation frame
        uvs[1] = SheetRegion::new(0, 16, 480, 8, 16, 16);
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
        trfs[2] = AABB {
            center: Vec2 {
                x: 48.0,
                y: 119.0,
            },
            size: Vec2 { x: 96.0, y: 131.0},
        }
        .into();
        uvs[2] = SheetRegion::new(0, 800, 448, 8, 96, 131);
        // set apple
        let mut apple_start = 3;
        for(target, (trf, uv)) in self.targets.iter().zip(
            trfs[apple_start..]
                .iter_mut()
                .zip(uvs[apple_start..].iter_mut()),
        ) {
            *trf = AABB {
                center: Vec2::new(target.pos.x - 88.0, target.pos.y),
                size: Vec2 { x: 16.0, y: 16.0 },
            }
            .into();
            match target.arrow_dir {
                0 => { *uv = SheetRegion::new(0, 0, 565, 4, 32, 32); }
                1 => { *uv = SheetRegion::new(0, 33, 565, 4, 32, 32); }
                2 => { *uv = SheetRegion::new(0, 66, 565, 4, 32, 32); }
                3 => { *uv = SheetRegion::new(0, 99, 565, 4, 32, 32); }
                _ => {}
            }
        }
        apple_start = apple_start + self.targets.len();

        // Compacting this to counter a really oddly specific bug.
        let mut arrowset: Vec<ArrowData> = Vec::new();
        for arrow in self.arrows0.iter() {
            arrowset.push(*arrow);
        }
        for arrow in self.arrows1.iter() {
            arrowset.push(*arrow);
        }
        for arrow in self.arrows2.iter() {
            arrowset.push(*arrow);
        }
        for arrow in self.arrows3.iter() {
            arrowset.push(*arrow);
        }

        for(arrow, (trf, uv)) in arrowset.iter().zip(
            trfs[apple_start..]
                .iter_mut()
                .zip(uvs[apple_start..].iter_mut()),
        ) {
            *trf = AABB {
                center: Vec2::new(arrow.start_pos.x - 88.0, arrow.start_pos.y + self.current_measure as f32 * 64.0),
                size: Vec2 { x: 4.0, y: 4.0 },
            }
            .into();
            match arrow.arrow_dir {
                0 => { *uv = SheetRegion::new(0, 0, 565, 4, 32, 32); }
                1 => { *uv = SheetRegion::new(0, 33, 565, 4, 32, 32); }
                2 => { *uv = SheetRegion::new(0, 66, 565, 4, 32, 32); }
                3 => { *uv = SheetRegion::new(0, 99, 565, 4, 32, 32); }
                _ => {}
            }
        }
        apple_start += arrowset.len();
        let sprite_count = apple_start;
        let score_str: String = "".to_string();
        let text_len = score_str.len();
        engine.renderer.sprites.resize_sprite_group(
            &engine.renderer.gpu,
            0,
            sprite_count + text_len,
        );
        self.font.draw_text(
            &mut engine.renderer.sprites,
            0,
            sprite_count,
            &score_str,
            Vec2 {
                x: 16.0,
                y: H - 16.0,
            }
            .into(),
            14.0,
        );
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..sprite_count + text_len);
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
    }
}

fn spawn_arrow (game: &mut Game, dir: usize, mouse_y: f64) {
    let mut arrow_y_val = ((mouse_y - 600.0) as f32).abs() * (240.0 / 600.0);
    arrow_y_val = ((arrow_y_val) / 4.0).round() * 4.0;
    arrow_y_val = arrow_y_val - 64.0 * game.current_measure as f32;
    let underbar = (arrow_y_val - 184.0).abs();

    let measure = (underbar / 64.0) as usize;
    let beat = ((underbar % 64.0) / 16.0) as usize;
    let sixteenth = ((underbar % 16.0) / 4.0) as usize;

    let target_time = TemporalMarker { 
        measure: measure, 
        beat: beat, 
        sixteenth: sixteenth as usize,
    };

    let arrow = ArrowData {
        start_pos: Vec2::new(dir as f32 * 24.0 + 100.0, arrow_y_val),
        travel_time: 300,
        arrow_dir: dir,
        target_time: target_time,
    };

    match dir {
        0 => {
            let initlen = game.arrows0.len();
            game.arrows0.retain(|i| i.start_pos.y as i32 != arrow_y_val as i32);
            if game.arrows0.len() == initlen {
                game.arrows0.push(arrow);
                game.arrows0.sort_by_key(|a| a.location());
            }
        }
        1 => {
            let initlen = game.arrows1.len();
            game.arrows1.retain(|i| i.start_pos.y as i32 != arrow_y_val as i32);
            if game.arrows1.len() == initlen {
                game.arrows1.push(arrow);
                game.arrows1.sort_by_key(|a| a.location());
            }
        }
        2 => {
            let initlen = game.arrows2.len();
            game.arrows2.retain(|i| i.start_pos.y as i32 != arrow_y_val as i32);
            if game.arrows2.len() == initlen {
                game.arrows2.push(arrow);
                game.arrows2.sort_by_key(|a| a.location());
            }
        }
        3 => {
            let initlen = game.arrows3.len();
            game.arrows3.retain(|i| i.start_pos.y as i32 != arrow_y_val as i32);
            if game.arrows3.len() == initlen {
                game.arrows3.push(arrow);
                game.arrows3.sort_by_key(|a| a.location());
            }
        }
        _ => {}
    }
}

fn export_stage(game: &mut Game) {
    let mut arrowset: Vec<ArrowData> = Vec::new();
    for arrow in game.arrows0.iter() {
        arrowset.push(*arrow);
    }
    for arrow in game.arrows1.iter() {
        arrowset.push(*arrow);
    }
    for arrow in game.arrows2.iter() {
        arrowset.push(*arrow);
    }
    for arrow in game.arrows3.iter() {
        arrowset.push(*arrow);
    }
    arrowset.sort_by_key(|a| a.location() as i32);

    std::fs::File::create("content/levels/".to_owned() + &game.output_filepath + ".rchart").unwrap();

    let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open("content/levels/".to_owned() + &game.output_filepath + ".rchart").unwrap();
    file.write(("Song, content/levels/".to_owned() + &game.song).as_bytes()).unwrap();
    file.write(("\nBPM, ".to_owned() + &game.bpm.to_string() + ", " + &game.num_beats.to_string()).as_bytes()).unwrap();
    for arrow in arrowset {
        let target_time = ((14400.0 / (4.0 * game.bpm as f64)) * (arrow.target_time.sixteenth + 4 * arrow.target_time.beat + 16 * arrow.target_time.measure) as f64).round() as i32;

        file.write((
            "\nArrow, ".to_owned() + 
            &(match arrow.arrow_dir {0 => {100.0} 1 => {150.0} 2 => {200.0} 3 => {250.0} _ => {300.0}}).to_string() + ", 0.0, " +
            &(match arrow.arrow_dir {0 => {100.0} 1 => {150.0} 2 => {200.0} 3 => {250.0} _ => {300.0}}).to_string() + ", 200.0, " +
            &arrow.arrow_dir.to_string() + ", " +
            &(target_time - arrow.travel_time).to_string() + ", " +
            &target_time.to_string() + ", " +
            "0.0, 0.0, 0.0, content/arrows.png, 0.0, 0.0, 0.1667, 0.1667"
        ).as_bytes()).unwrap();
    }
    file.flush().unwrap();
}

fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
