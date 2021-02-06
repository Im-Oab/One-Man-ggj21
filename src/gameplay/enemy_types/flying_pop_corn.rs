use std::collections::HashMap;
use std::time::Duration;

use rand::prelude::*;

use tetra::math::Vec2;
use tetra::Context;

use crate::image_assets::ImageAssets;
use crate::sprite::AnimationMultiTextures;

use crate::gameplay::enemy_manager::{Enemy, EnemyType};
use crate::gameplay::player::Player;

pub struct FlyingPopCornEnemyType {
    _animations: HashMap<String, AnimationMultiTextures>,
}

impl FlyingPopCornEnemyType {
    #[must_use]
    pub fn new(image_assets: &ImageAssets) -> Self {
        let mut animations = HashMap::new();

        let keys = [""];

        for &key in &keys {
            if let Some(mut anim) = image_assets.get_animation_object(key) {
                anim.frame_length = Duration::from_millis(1000 / 15);

                animations.insert(key.to_owned(), anim);
            } else {
                println!("No animation name: {}", key)
            };
        }

        Self {
            _animations: animations,
        }
    }

    fn random_target_position(enemy: &mut Enemy) {
        enemy.target_position.clear();
        for _ in 0..3 {
            // enemy.target_position.push(if enemy.state % 2 == 0 {
            //     crate::gameplay::utils::random_position_inside_camera_area(0.1, 0.1, 0.3, 0.3)
            // } else {
            //     crate::gameplay::utils::random_position_inside_camera_area(0.6, 0.1, 0.3, 0.3)
            // });

            enemy
                .target_position
                .push(crate::gameplay::utils::random_position_inside_camera_area(
                    0.1, 0.1, 0.8, 0.4,
                ));
        }
    }

    fn random_weapon_tick() -> u128 {
        2500 + (random::<f32>() * 1000.0) as u128
    }
}

impl EnemyType for FlyingPopCornEnemyType {
    fn enemy_type_id(&self) -> i32 {
        1
    }

    fn init(&mut self, enemy: &mut Enemy, image_assets: &ImageAssets) {
        enemy.enemy_type = self.enemy_type_id();
        enemy.radius = 8.0;
        enemy.active = true;
        enemy.health = 5;
        enemy.life_time = 100;
        enemy.maximum_tick = 3000;
        enemy.weapon_tick = Self::random_weapon_tick();
        if enemy.position.x
            < crate::gameplay::utils::convert_screen_position_to_world_position(Vec2::new(0.0, 0.0))
                .x
        {
            if enemy.state % 2 == 1 {
                enemy.state += 1;
            }
        } else if enemy.state % 2 == 0 {
            enemy.state += 1;
        }

        if let Some(animation) = image_assets.get_animation_object("enemy-flying-spawn") {
            enemy.sprite.scale = Vec2::new(1.4, 1.4);
            enemy.sprite.set_loop(false);
            enemy.sprite.play(&animation);
        };

        Self::random_target_position(enemy);
    }

    fn update(&self, enemy: &mut Enemy, player: Option<&Player>, image_assets: &ImageAssets) {
        enemy.tick = enemy
            .tick
            .checked_add(crate::ONE_FRAME.as_millis())
            .unwrap_or(0);

        enemy.weapon_tick = enemy
            .weapon_tick
            .saturating_sub(crate::ONE_FRAME.as_millis());

        if enemy.weapon_tick == 0 {
            enemy.weapon_tick = Self::random_weapon_tick();
            Enemy::spawn_bullet(
                enemy.position,
                player.unwrap().get_hit_point_position(),
                1,
                1.0,
                4.0,
                "idle_animation=enemy-bullet-1-idle|firing_animation=enemy-bullet-1-firing|hit_animation=enemy-bullet-1-hit|kill_animation=enemy-bullet-1-kill|scale=1.2|",
            );
        }

        if enemy.sprite.get_current_animation_name() == "enemy-flying-spawn"
            && enemy.sprite.is_end_of_animation()
        {
            if let Some(animation) = image_assets.get_animation_object("enemy-flying-idle") {
                enemy.sprite.set_loop(true);
                enemy.sprite.play(&animation);
            };
        }

        if enemy.tick > enemy.maximum_tick {
            enemy.state += 1;
            enemy.tick = 0;
            Self::random_target_position(enemy);
        }

        let mut actual_position = enemy.position;
        for position in &enemy.target_position {
            actual_position = Vec2::lerp(actual_position, *position, 0.01);
        }
        enemy.position = actual_position;
    }

    fn draw(&self, ctx: &mut Context, image_assets: &ImageAssets, enemy: &mut Enemy) {
        enemy.sprite.draw(ctx, enemy.position, 0.0, image_assets);
    }

    /// This function will called internally in die().
    /// It will decide that what it should do with the bullets on screen when this enemy die
    fn die(&self, _enemy: &mut Enemy) {
        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(
                String::from("flying_explode"),
                (String::from("./resources/sfx/flying_explode.mp3"), 0.6),
            );
        }
    }

    /// Return: 0: not hit, 1: hit weakpoint, -1: hit shield. (No damage)
    fn hit_check(&self, enemy: &Enemy, position: Vec2<f32>, radius: f32) -> i32 {
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
