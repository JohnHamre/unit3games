// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::{self as engine, load_stage};
use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use engine::structs::*;
//use rand::Rng;
pub use engine::structs::Arrow;
pub use std::collections::VecDeque;
use std::process::exit;
const W: f32 = 320.0;
const H: f32 = 240.0;
const SPRITE_MAX: usize = 16;
const CATCH_DISTANCE: f32 = 4.0;
const COLLISION_STEPS: usize = 3;

struct Guy {
    pos: Vec2,
}

struct GameState {
    walls: Vec<AABB>,
    guy: Guy,
    arrows: Vec<Arrow>,
    apple_timer: u32,
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
        let guy = Guy {
            pos: Vec2 {
                x: 150.0,
                y: 24.0,
            },
        };
        let floor = AABB {
            center: Vec2 { x: W / 2.0, y: 8.0 },
            size: Vec2 { x: W, y: 16.0 },
        };
        let left_wall = AABB {
            center: Vec2 { x: 8.0, y: H / 2.0 },
            size: Vec2 { x: 16.0, y: H },
        };
        let right_wall = AABB {
            center: Vec2 {
                x: W - 8.0,
                y: H / 2.0,
            },
            size: Vec2 { x: 16.0, y: H },
        };

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 220, 464, 12, 70, 10),
            10,
        );
        Game {
            camera,
            gamestate: GameState {
                guy,
                walls: vec![left_wall, right_wall, floor],
                arrows: Vec::with_capacity(16),
                apple_timer: 0,
                score: 3,
            },
            font,
        }
    }
    fn update(&mut self, engine: &mut Engine, frame_events: &mut VecDeque<REvent>) {
        if engine.input.is_key_pressed(engine::Key::Left) && self.gamestate.guy.pos.x > 110.0 {
            self.gamestate.guy.pos.x -= 50.0;
        }
        if engine.input.is_key_pressed(engine::Key::Right) && self.gamestate.guy.pos.x < 240.0 {
            self.gamestate.guy.pos.x += 50.0;
        }
        if engine.input.is_key_pressed(engine::Key::R) {
            load_stage(engine, "content/levels/bill_nye.rchart");
            self.gamestate.arrows.clear();
            self.gamestate.score = 3;
        }
        for event in frame_events.drain(..) {
            spawn_event(self, event);
        }
        let mut contacts = Vec::with_capacity(self.gamestate.walls.len());
        // TODO: for multiple guys this might be better as flags on the guy for what side he's currently colliding with stuff on
        for _iter in 0..COLLISION_STEPS {
            let guy_aabb = AABB {
                center: self.gamestate.guy.pos,
                size: Vec2 { x: 16.0, y: 1.0 },
            };
            contacts.clear();
            // TODO: to generalize to multiple guys, need to iterate over guys first and have guy_index, rect_index, displacement in a contact tuple
            contacts.extend(
                self.gamestate.walls
                    .iter()
                    .enumerate()
                    .filter_map(|(ri, w)| w.displacement(guy_aabb).map(|d| (ri, d))),
            );
            if contacts.is_empty() {
                break;
            }
            // This part stays mostly the same for multiple guys, except the shape of contacts is different
            contacts.sort_by(|(_r1i, d1), (_r2i, d2)| {
                d2.length_squared()
                    .partial_cmp(&d1.length_squared())
                    .unwrap()
            });
            for (wall_idx, _disp) in contacts.iter() {
                // TODO: for multiple guys should access self.guys[guy_idx].
                let guy_aabb = AABB {
                    center: self.gamestate.guy.pos,
                    size: Vec2 { x: 16.0, y: 1.0 },
                };
                let wall = self.gamestate.walls[*wall_idx];
                let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                // We got to a basically zero collision amount
                if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                    break;
                }
                // Guy is left of wall, push left
                if self.gamestate.guy.pos.x < wall.center.x {
                    disp.x *= -1.0;
                }
                // Guy is below wall, push down
                if self.gamestate.guy.pos.y < wall.center.y {
                    disp.y *= -1.0;
                }
                if disp.x.abs() <= disp.y.abs() {
                    self.gamestate.guy.pos.x += disp.x;
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                } else if disp.y.abs() <= disp.x.abs() {
                    self.gamestate.guy.pos.y += disp.y;
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                }
            }
        }
        //let mut rng = rand::thread_rng();
        if self.gamestate.apple_timer > 0 {
            self.gamestate.apple_timer -= 1;
        } else if self.gamestate.arrows.len() < 8 {
            // spawn_arrow( 
            //     self,
            //     Vec2 {
            //         //x: rng.gen_range(8.0..(W - 8.0)),
            //         x: W/2.0,
            //         y: H + 8.0,
            //     },
            //     Vec2 {
            //         x: 0.0,
            //         //y: rng.gen_range((-4.0)..(-1.0)),
            //         y: -1.0,
            //     },
            // 0.0,0.0,1,0);
            self.gamestate.apple_timer = 100;
        }
        for arrow in self.gamestate.arrows.iter_mut() {
            arrow.pos += arrow.vel;
        }
        if let Some(idx) = self.gamestate
            .arrows
            .iter()
            .position(|arrow: &Arrow| arrow.pos.distance(self.gamestate.guy.pos) <= CATCH_DISTANCE)
        {
            self.gamestate.arrows.swap_remove(idx);
            self.gamestate.score -= 1;
            if self.gamestate.score <= 0 {
                println!("GAME OVER!");
                exit(0);
            }
        }
        self.gamestate.arrows.retain(|apple| apple.pos.y > -8.0)
    }
    fn render(&mut self, engine: &mut Engine) {
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
        // set 4 target arrows
        // set bg
        trfs[0] = AABB {
            center: Vec2 {
                x: W / 2.0,
                y: H / 2.0,
            },
            size: Vec2 { x: W, y: H },
        }
        .into();
        uvs[0] = SheetRegion::new(0, 0, 0, 16, 880, 440);
        // set walls
        const WALL_START: usize = 1;
        let guy_idx = WALL_START + self.gamestate.walls.len();
        for (wall, (trf, uv)) in self.gamestate.walls.iter().zip(
            trfs[WALL_START..guy_idx]
                .iter_mut()
                .zip(uvs[WALL_START..guy_idx].iter_mut()),
        ) {
            *trf = (*wall).into();
            *uv = SheetRegion::new(0, 0, 480, 12, 0, 0);
        }
        // set guy
        trfs[guy_idx] = AABB {
            center: self.gamestate.guy.pos,
            size: Vec2 { x: 16.0, y: 16.0 },
        }
        .into();
        // TODO animation frame
        uvs[guy_idx] = SheetRegion::new(0, 16, 480, 8, 16, 16);
        // set apple
        let apple_start = guy_idx + 1;
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
    let mut a = arrow;
    a.vel.y = -a.vel.y;
    a.pos.y = H + -a.vel.y * (a.target_time - a.start_time) as f32;
    game.gamestate.arrows.push(a);
}

fn spawn_event(game: &mut Game, event: REvent) {
    match event {
        REvent::ArrowEvent(a) => {
            spawn_arrow(game, a);
        }
        _ => {}
    }
}

fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
