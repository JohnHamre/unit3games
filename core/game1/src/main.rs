// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

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
const MERCIES: (i32, i32, i32) = (10, 20, 30);

struct GameState {
    arrows: Vec<Arrow>,
    score: u32,
}

struct Game {
    camera: engine::Camera,
    gamestate: GameState,
    font: engine::BitFont,
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
        Game {
            camera,
            gamestate: GameState {
                arrows: Vec::with_capacity(16),
                score: 0,
            },
            font,
        }
    }
    fn update(&mut self, engine: &mut Engine, frame_events: &mut VecDeque<REvent>) {
        // Input catcher for each arrow type
        if  engine.input.is_key_pressed(engine::Key::A) || 
            engine.input.is_key_pressed(engine::Key::Left) {
                update_score(self, engine, 0);
        }

        if  engine.input.is_key_pressed(engine::Key::S) || 
        engine.input.is_key_pressed(engine::Key::Down) {
            update_score(self, engine, 1);
        }

        if  engine.input.is_key_pressed(engine::Key::W) || 
        engine.input.is_key_pressed(engine::Key::Up) {
            update_score(self, engine, 2);
        }

        if  engine.input.is_key_pressed(engine::Key::D) || 
        engine.input.is_key_pressed(engine::Key::Right) {
            update_score(self, engine, 3);
        }

        // Spawn new events per frame.
        for event in frame_events.drain(..) {
            spawn_event(self, event);
        }
        // Move arrows per frame.
        for arrow in self.gamestate.arrows.iter_mut() {
            arrow.pos += arrow.vel;
        }
        // Clear arrows that have left the world
        let initlen = self.gamestate.arrows.len();
        self.gamestate.arrows.retain(|apple| apple.pos.y < H + 8.0);
        if initlen > self.gamestate.arrows.len() {
            for _ in 0..(initlen - self.gamestate.arrows.len())
            {
                self.gamestate.score = subtract_with_floor(self.gamestate.score, 500);
            }
        }
    }
    fn render(&mut self, engine: &mut Engine) {
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
        // set 4 target arrows
        trfs[0] = AABB {
            center: Vec2 {
                x: 100.0,
                y: 200.0,
            },
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        uvs[0] = SheetRegion::new(0, 0, 464, 16, 32, 32);
        trfs[1] = AABB {
            center: Vec2 {
                x: 150.0,
                y: 200.0,
            },
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        uvs[1] = SheetRegion::new(0, 34, 464, 16, 32, 32); 
        trfs[2] = AABB {
            center: Vec2 {
                x: 200.0,
                y: 200.0,
            },
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        uvs[2] = SheetRegion::new(0, 66, 464, 16, 32, 32);
        trfs[3] = AABB {
            center: Vec2 {
                x: 250.0,
                y: 200.0,
            },
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        uvs[3] = SheetRegion::new(0, 99, 464, 16, 32, 32);
        // set bg
        trfs[4] = AABB {
            center: Vec2 {
                x: W / 2.0,
                y: H / 2.0,
            },
            size: Vec2 { x: W, y: H },
        }
        .into();
        uvs[4] = SheetRegion::new(0, 0, 0, 16, 880, 440);
        // set apple
        let apple_start = 5;
        for (apple, (trf, uv)) in self.gamestate.arrows.iter().zip(
            trfs[apple_start..]
                .iter_mut()
                .zip(uvs[apple_start..].iter_mut()),
        ) {
            *trf = AABB {
                center: apple.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            }
            .into();
            match apple.arrow_dir {
                0 => { *uv = SheetRegion::new(0, 0, 565, 4, 32, 32); }
                1 => { *uv = SheetRegion::new(0, 34, 565, 4, 32, 32); }
                2 => { *uv = SheetRegion::new(0, 66, 565, 4, 32, 32); }
                3 => { *uv = SheetRegion::new(0, 99, 565, 4, 32, 32); }
                _ => {}
            }
        }
        let sprite_count = apple_start + self.gamestate.arrows.len();
        let score_str = self.gamestate.score.to_string();
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

fn spawn_arrow(game: &mut Game, arrow: Arrow) {
    game.gamestate.arrows.push(arrow);
}

fn spawn_event(game: &mut Game, event: REvent) {
    match event {
        REvent::ArrowEvent(a) => {
            spawn_arrow(game, a);
        }
        _ => {}
    }
}

fn update_score(game: &mut Game, engine: &mut Engine, arrow_dir: usize) {
    let indices0 = game.gamestate.arrows
    .iter()
    .enumerate()
    .filter(|(_, r)| 
        ((r.target_time - engine.level_state.queue_timer).abs() < MERCIES.0) && 
        (r.arrow_dir == arrow_dir)
    )
    .map(|(index, _)| index)
    .collect::<Vec<_>>();

    let indices1 = game.gamestate.arrows
    .iter()
    .enumerate()
    .filter(|(_, r)| 
        ((r.target_time - engine.level_state.queue_timer).abs() < MERCIES.1) && 
        (r.arrow_dir == arrow_dir)
    )
    .map(|(index, _)| index)
    .collect::<Vec<_>>();
    
    let indices2 = game.gamestate.arrows
    .iter()
    .enumerate()
    .filter(|(_, r)| 
        ((r.target_time - engine.level_state.queue_timer).abs() < MERCIES.2) && 
        (r.arrow_dir == arrow_dir)
    )
    .map(|(index, _)| index)
    .collect::<Vec<_>>();

    if indices0.len() > 0 {
        game.gamestate.arrows.remove(indices0[0]);
        game.gamestate.score += 1000;
    }
    else if indices1.len() > 0 {
        game.gamestate.arrows.remove(indices1[0]);
        game.gamestate.score += 500;
    }
    else if indices2.len() > 0 {
        game.gamestate.arrows.remove(indices2[0]);
        game.gamestate.score += 100;
    }
    else {
        game.gamestate.score = subtract_with_floor(game.gamestate.score, 500);
    }
}

fn subtract_with_floor(num: u32, amount: i32) -> u32 {
    return if num as i32 - amount < 0 {0} else {(num as i32 - amount) as u32};
}

fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
