use std::collections::HashMap;

use std::sync::Mutex;
use std::time::Duration;

use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::{self, Color};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};

use crate::gameplay::bullet_pool::{BulletPool, BulletSpawnNode, BulletTypeBank};

use crate::gameplay::enemy_manager::EnemyTypeBank;
use crate::gameplay::level::EnemySpawnNode;
use crate::gameplay::particle_manager::{ParticleSpawnNode, ParticleTypeBank};

use crate::scene::{Scene, Transition};
use crate::scenes::gameplay::GamePlayScene;

/// Game screen resolution : Width
pub const SCREEN_WIDTH: f32 = 480.0;
/// Game screen resolution : Height
pub const SCREEN_HEIGHT: f32 = 240.0;

pub const GROUND: f32 = 24.0;
pub const CEILING: f32 = SCREEN_HEIGHT * -0.7;
pub const GRAVITY: f32 = 4.0;

pub const ONE_FRAME: Duration = Duration::from_millis(1000 / 60);

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref CAMERA_POSITION: Mutex<Vec2<f32>> = Mutex::new(Vec2::zero());
    pub static ref BULLET_POOL: Mutex<BulletPool> = Mutex::new(BulletPool::new(500));
    pub static ref BULLET_SPAWN_NODES: Mutex<Vec<BulletSpawnNode>> = Mutex::new(vec![]);
    pub static ref BULLET_TYPE_BANK: Mutex<BulletTypeBank> = Mutex::new(BulletTypeBank::new());
    pub static ref ENEMY_TYPE_BANK: Mutex<EnemyTypeBank> = Mutex::new(EnemyTypeBank::new());
    pub static ref ENEMY_SPAWN_NODES: Mutex<Vec<EnemySpawnNode>> = Mutex::new(Vec::new());
    pub static ref PARTICLE_TYPE_BANK: Mutex<ParticleTypeBank> =
        Mutex::new(ParticleTypeBank::new());
    pub static ref PARTICLE_SPAWN_NODES: Mutex<Vec<ParticleSpawnNode>> = Mutex::new(Vec::new());
    pub static ref PLAY_SOUND_NODES: Mutex<HashMap<String, (String, f32)>> = Mutex::new(HashMap::new());
}

struct GameState {
    scenes: Vec<Box<dyn Scene>>,
    scaler: ScreenScaler,
}

pub mod image_assets;

pub mod scene;
pub mod scenes {
    pub mod gameplay;
}

pub mod sprite;

pub mod gameplay {
    pub mod bullet_pool;
    pub mod enemy_manager;
    pub mod input;
    pub mod level;
    pub mod particle_manager;
    pub mod player;
    pub mod ui;
    pub mod utils;

    pub mod bullet_types {
        pub mod constant_velocity;
    }

    pub mod enemy_types {
        pub mod crawling_pop_corn;
        pub mod flying_pop_corn;
        pub mod spawner;
        pub mod boss;
    }

    pub mod particle_types {
        pub mod explosion;
    }
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        // let initial_scene = second_scene::SecondScene::new(ctx)?;
        // let initial_scene = AnimationPreview::new(ctx)?;
        // let initial_scene = EnemySandboxScene::new()?;
        let initial_scene = GamePlayScene::new(ctx)?;

        Ok(GameState {
            scenes: vec![Box::new(initial_scene)],
            scaler: ScreenScaler::with_window_size(
                ctx,
                SCREEN_WIDTH as i32,
                SCREEN_HEIGHT as i32,
                ScalingMode::ShowAll,
            )?,
        })
    }
}

use tetra::window;

impl State for GameState {
    /// Call scene.update() and look for "Transition" result for chaning scene.
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        window::set_title(ctx, format!("One-Man: {}", tetra::time::get_fps(ctx) as i32));

        match self.scenes.last_mut() {
            Some(active_scene) => match active_scene.update(ctx)? {
                Transition::None => {}
                Transition::Push(s) => {
                    self.scenes.push(s);
                }
                Transition::Pop => {
                    self.scenes.pop();
                }
                Transition::Replace(s) => {
                    self.scenes.clear();
                    self.scenes.push(s)
                }
            },
            None => {}
        }

        Ok(())
    }

    /// Draw scene with scaling resolution to match aspect ratio and window size.
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());

        match self.scenes.last_mut() {
            Some(active_scene) => active_scene.draw(ctx),
            None => {
                panic!("No active scene")
            }
        }

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);
        graphics::draw(ctx, &self.scaler, Vec2::new(0.0, 0.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }

        Ok(())
    }
}

use tetra::time::Timestep;
/// Setup window size, window title, etc.
fn main() -> tetra::Result {
    let scale = 1;
    ContextBuilder::new(
        "One man",
        crate::SCREEN_WIDTH as i32 / scale,
        crate::SCREEN_HEIGHT as i32 / scale,
    )
    .resizable(true)
    .maximized(false)
    .quit_on_escape(true)
    .timestep(Timestep::Fixed(60.0))
    .borderless(false)
    .fullscreen(false)
    .build()?
    .run(GameState::new)
}
