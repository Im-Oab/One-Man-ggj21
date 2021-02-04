use std::collections::HashMap;
use std::time::Duration;

use rand::prelude::*;

use tetra::graphics::{self, Color, GeometryBuilder, Mesh, Rectangle, ShapeStyle};
use tetra::math::Vec2;
use tetra::Context;

use crate::image_assets::ImageAssets;
use crate::sprite::AnimationMultiTextures;

use crate::gameplay::enemy_manager::{Enemy, EnemyManager, EnemyType};
use crate::gameplay::level::EnemySpawnNode;
use crate::gameplay::player::Player;

pub struct SpawnerEnemyType {
    animations: HashMap<String, AnimationMultiTextures>,
}
/// spawn_time, spawn_interval, spawn_queue
impl SpawnerEnemyType {
    pub fn new(image_assets: &ImageAssets) -> SpawnerEnemyType {
        let mut animations = HashMap::new();

        SpawnerEnemyType {
            animations: animations,
        }
    }

    fn spawn_tick(&self, enemy: &mut Enemy) {
        if enemy.tick == 0 {
            enemy.tick = match enemy.extra.get("spawn_interval") {
                Some(v) => v.parse::<u128>().unwrap_or(0),
                None => 0,
            };

            let spawn_enemy_type_id = match enemy.extra.get_mut("spawn_queue") {
                Some(v) => {
                    if v.len() != 0 {
                        let first_letter = v.chars().nth(0).unwrap();
                        SpawnerEnemyType::crop_letters(v, 1);
                        String::from(first_letter).parse::<i32>().unwrap_or(0)
                    } else {
                        0
                    }
                }
                None => {
                    println!("No spawn_queue key");
                    0
                }
            };

            if spawn_enemy_type_id != 0 {
                let mut need_to_spawn_enemy_list = crate::ENEMY_SPAWN_NODES.lock().unwrap();
                need_to_spawn_enemy_list.push(EnemySpawnNode::new(
                    0,
                    spawn_enemy_type_id,
                    enemy.position
                        + Vec2::new(
                            8.0 - random::<f32>() * 16.0,
                            (8.0 - random::<f32>() * 16.0) - 24.0,
                        ),
                    "",
                ));

                {
                    let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                    play_sound_nodes.insert(String::from("spawner"), (String::from("./resources/sfx/spawner.mp3"), 0.4 ) );
                }

                enemy.spawn_splash(enemy.position, 0.5);
                enemy.spawn_splash(enemy.position, 0.5);
                enemy.spawn_splash(enemy.position, 0.5);
            } else {
                enemy.active = false;
            }
        } else {
            match enemy.tick.checked_sub(crate::ONE_FRAME.as_millis()) {
                Some(v) => enemy.tick = v,
                None => enemy.tick = 0,
            };
        }
    }

    fn crop_letters(s: &mut String, pos: usize) {
        match s.char_indices().nth(pos) {
            Some((pos, _)) => {
                s.drain(..pos);
            }
            None => {
                s.clear();
            }
        }
    }
}

impl EnemyType for SpawnerEnemyType {
    fn enemy_type_id(&self) -> i32 {
        0
    }

    fn init(&mut self, enemy: &mut Enemy, image_assets: &ImageAssets) {
        enemy.enemy_type = self.enemy_type_id();
        enemy.radius = 32.0;
        enemy.active = true;
        enemy.health = 100000;
        enemy.life_time = 100;
        enemy.maximum_tick = 3000;

        enemy.weapon_tick = match enemy.extra.get("spawn_time") {
            Some(v) => v.parse::<u128>().unwrap_or(123456),
            None => 123456,
        };

        match enemy.extra.get("scale") {
            Some(v) => {
                let scale = v.parse::<f32>().unwrap_or(2.5);
                enemy.sprite.scale = Vec2::new(scale, scale);
            }
            None => {
                enemy.sprite.scale = Vec2::new(2.5, 2.5);
            }
        }

        match enemy.extra.get("flip_x") {
            Some(v) => {
                if v == "1" {
                    enemy.sprite.flip_x(true);
                } else {
                    enemy.sprite.flip_x(false);
                }
            }
            None => {}
        }

        match enemy.extra.get("idle_animation") {
            Some(animation_name) => {
                match image_assets.get_animation_object(animation_name.as_str()) {
                    Some(animation) => {
                        enemy.sprite.play(&animation);
                    }
                    None => {
                        enemy.active = false;
                    }
                }
            }
            None => enemy.active = false,
        };
    }

    fn update(&self, enemy: &mut Enemy, player: Option<&Player>, image_assets: &ImageAssets) {
        if enemy.weapon_tick != 123456 {
            match enemy.weapon_tick.checked_sub(crate::ONE_FRAME.as_millis()) {
                Some(v) => enemy.weapon_tick = v,
                None => {
                    enemy.weapon_tick = 0;
                }
            };
        } else {
            enemy.health = 0;
        }

        if enemy.weapon_tick == 0 {
            match enemy.extra.get("spawning_animation") {
                Some(animation_name) => {
                    if enemy.sprite.get_current_animation_name() != animation_name {
                        match image_assets.get_animation_object(animation_name.as_str()) {
                            Some(animation) => {
                                enemy.sprite.play(&animation);
                            }
                            None => {}
                        }
                    }
                }
                None => (),
            };

            self.spawn_tick(enemy);
        }
    }

    fn draw(&self, ctx: &mut Context, image_assets: &ImageAssets, enemy: &mut Enemy) {
        enemy.sprite.draw(ctx, enemy.position, 0.0, image_assets);
    }

    fn die(&self, enemy: &mut Enemy) {
        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("spawner_explode"), (String::from("./resources/sfx/spawner_explode.mp3"), 0.8 ) );
        }
    }

    /// 0: not hit, 1: hit weakpoint, -1: hit shield. (No damage)
    fn hit_check(&self, enemy: &Enemy, position: &Vec2<f32>, radius: f32) -> i32 {
        0
    }
}
