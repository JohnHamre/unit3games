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

struct GameState {
    arrows: Vec<Arrow>,
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
        Game {
            camera,
            gamestate: GameState {
                arrows: Vec::with_capacity(16),
            },
            font,
        }
    }
    fn update(&mut self, _engine: &mut Engine, _frame_events: &mut VecDeque<REvent>) {
        //let dir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        for _arrow in self.gamestate.arrows.iter_mut() {
            // arrow.pos += arrow.vel;
        }
        self.gamestate.arrows.retain(|apple| apple.pos.y > -8.0)
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
        let apple_start = 1 + 1;
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
                1 => { *uv = SheetRegion::new(0, 33, 565, 4, 32, 32); }
                2 => { *uv = SheetRegion::new(0, 66, 565, 4, 32, 32); }
                3 => { *uv = SheetRegion::new(0, 99, 565, 4, 32, 32); }
                _ => {}
            }
        }
        let sprite_count = apple_start + self.gamestate.arrows.len();
        let score_str = "0123".to_string();
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

fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
