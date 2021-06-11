use std::collections::HashMap;
use std::time::Duration;

use rand::prelude::*;

use tetra::graphics::{self, Color, GeometryBuilder, Mesh, Rectangle, ShapeStyle};
use tetra::math::Vec2;
use tetra::Context;

use crate::image_assets::ImageAssets;
use crate::sprite::AnimationMultiTextures;

use crate::gameplay::enemy_manager::{Enemy, EnemyType};
use crate::gameplay::player::Player;

pub struct CrawlingPopCornEnemyType {}

impl CrawlingPopCornEnemyType {
    pub fn new(image_assets: &ImageAssets) -> CrawlingPopCornEnemyType {
        CrawlingPopCornEnemyType {}
    }

    fn apply_gravity(&self, enemy: &mut Enemy, offset: f32) {
        enemy.position.y += crate::GRAVITY * offset;
        enemy.position.y = enemy.position.y.min(0.0);
    }

    fn on_the_ground(&self, enemy: &mut Enemy) -> bool {
        enemy.position.y >= -10.0
    }

    fn random_weapon_tick(&self, enemy: &mut Enemy) {
        enemy.weapon_tick = 800 + random::<u128>() % 800;
    }

    fn update_rotation(&self, enemy: &mut Enemy, player_position: Vec2<f32>) {
        if enemy.position.x < player_position.x {
            enemy.rotation = 0.05;
        } else {
            enemy.rotation = 0.45;
        }
    }
}

impl EnemyType for CrawlingPopCornEnemyType {
    fn enemy_type_id(&self) -> i32 {
        2
    }

    fn init(&mut self, enemy: &mut Enemy, image_assets: &ImageAssets) {
        enemy.enemy_type = self.enemy_type_id();
        enemy.radius = 20.0;
        enemy.active = true;
        enemy.health = 40;
        enemy.life_time = 1000;
        enemy.sprite.scale = Vec2::new(2.0, 2.0);

        enemy.rotation = match enemy.extra.get("rotation") {
            Some(v) => v.parse::<f32>().unwrap_or(0.0),
            None => 0.0,
        };

        self.random_weapon_tick(enemy);

        match image_assets.get_animation_object("enemy-crawler-air") {
            Some(animation) => {
                enemy.sprite.play(&animation);
            }
            None => {
                enemy.active = false;
            }
        }
    }

    fn update(&self, enemy: &mut Enemy, player: Option<&Player>, image_assets: &ImageAssets) {
        if enemy.state == 0 {
            let speed = 4.0;
            if enemy.weapon_tick <= 500 {
                self.apply_gravity(enemy, 0.5 * (1.0 - (enemy.weapon_tick as f32 / 500.0)));

                
            } else {
                enemy.position.y -= (enemy.rotation * 360.0).to_radians().sin() * speed;
            }

            if self.on_the_ground(enemy) {
                enemy.state = 1;
                enemy.weapon_tick = 1500;
                self.update_rotation(enemy, player.unwrap().get_hit_point_position());
                match image_assets.get_animation_object("enemy-crawler-idle") {
                    Some(animation) => {
                        enemy.sprite.play(&animation);
                    }
                    None => {
                        enemy.active = false;
                    }
                }
            }

            enemy.position.x += (enemy.rotation * 360.0).to_radians().cos() * speed;
        } else {
            let speed = 3.0;
            enemy.position.x += (enemy.rotation * 360.0).to_radians().cos()
                * (speed * (enemy.weapon_tick as f32 / 1500.0));

            if enemy.weapon_tick == 0 {
                enemy.weapon_tick = 1500;
                self.update_rotation(enemy, player.unwrap().get_hit_point_position());
            }
        }

        match enemy.weapon_tick.checked_sub(crate::ONE_FRAME.as_millis()) {
            Some(v) => enemy.weapon_tick = v,
            None => enemy.weapon_tick = 0,
        };

        if enemy.rotation > 0.25 && enemy.rotation < 0.75 {
            enemy.sprite.flip_x(true);
        } else {
            enemy.sprite.flip_x(false);
        }
    }

    fn draw(&self, ctx: &mut Context, image_assets: &ImageAssets, enemy: &mut Enemy) {

        enemy.sprite.draw(
            ctx,
            enemy.position + Vec2::new(0.0, -8.0),
            0.0,
            image_assets,
        );
    }

    fn die(&self, enemy: &mut Enemy) {
        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("crawl_explode"), (String::from("./resources/sfx/crawl_explode.mp3"), 0.8 ) );
        }
    }

    /// 0: not hit, 1: hit weakpoint, -1: hit shield. (No damage)
    fn hit_check(&self, enemy: &Enemy, position: &Vec2<f32>, radius: f32) -> i32 {
        let distance = crate::gameplay::utils::distance_sqr(
            enemy.position.x as i128,
            enemy.position.y as i128,
            position.x as i128,
            position.y as i128,
        );
        let total_radius = (enemy.radius + radius) as i128;
        if distance <= total_radius * total_radius {
            return 1;
        }

        0
    }
}
