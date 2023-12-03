// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size
use std::fs;
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

#[derive(Zeroable, Debug, Copy, Clone)]
struct ArrowData {
    start_pos: Vec2,
    time: i32,
    arrow_dir: usize,
}

struct Game {
    camera: engine::Camera,
    targets: Vec<TargetData>,
    font: engine::BitFont,
    arrows0: Vec<ArrowData>,
    arrows1: Vec<ArrowData>,
    arrows2: Vec<ArrowData>,
    arrows3: Vec<ArrowData>,
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
                    println!("Name: {}", p_str);
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
        }
    }
    fn update(&mut self, engine: &mut Engine, _frame_events: &mut VecDeque<REvent>) {
        //let dir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        let mouse_pos = engine.input.mouse_pos();
        let mut col = 4;
        if mouse_pos.y >= 128.0 {
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
        // set apple
        let mut apple_start = 1 + 1;
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
                center: Vec2::new(arrow.start_pos.x - 88.0, arrow.start_pos.y),
                size: Vec2 { x: 16.0, y: 16.0 },
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
    let arrow = ArrowData {
        start_pos: Vec2::new(dir as f32 * 24.0 + 100.0, ((mouse_y - 600.0) as f32).abs() * (240.0 / 600.0)),
        time: 300,
        arrow_dir: dir,
    };
    match dir {
        0 => {
            game.arrows0.push(arrow);
            game.arrows0.sort_by_key(|a| a.start_pos.y as i32);
        }
        1 => {
            game.arrows1.push(arrow);
            game.arrows1.sort_by_key(|a| a.start_pos.y as i32);
        }
        2 => {
            game.arrows2.push(arrow);
            game.arrows2.sort_by_key(|a| a.start_pos.y as i32);
        }
        3 => {
            game.arrows3.push(arrow);
            game.arrows3.sort_by_key(|a| a.start_pos.y as i32);
        }
        _ => {}
    }
}

fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
