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
use crate::gameplay::level::EnemySpawnNode;

pub struct BossEnemyType {
}

impl BossEnemyType {
    pub fn new(image_assets: &ImageAssets) -> BossEnemyType {
        

        BossEnemyType {
            
        }
    }

    fn random_target_position(&self, enemy: &mut Enemy,) {
        enemy.target_position.clear();
        for _ in 0..3 {
            enemy
                .target_position
                .push(crate::gameplay::utils::random_position_inside_camera_area(
                    0.1, 0.1, 0.8, 0.4,
                ));
        }
    }

    fn random_weapon_tick(&self) -> u128 {
        2500 + (random::<f32>() * 1000.0) as u128
    }

    fn do_action(&self, enemy: &mut Enemy, player: Option<&Player>)
    {
        if enemy.state == 1 && enemy.weapon_tick == 0
        {
            let mut need_to_spawn_enemy_list = crate::ENEMY_SPAWN_NODES.lock().unwrap();

            let spawn_position = enemy.position
            + Vec2::new(
                8.0 - random::<f32>() * 16.0,
                (8.0 - random::<f32>() * 16.0) + 48.0,
            );

            if random::<u8>() % 3 == 0
            {
                need_to_spawn_enemy_list.push(EnemySpawnNode::new(
                    0,
                    2,
                    spawn_position,
                    format!("rotation={}|", random::<f32>()).as_str(),
                ));
            }
            else
            {
                need_to_spawn_enemy_list.push(EnemySpawnNode::new(
                    0,
                    1,
                    spawn_position,
                    format!("rotation={}|", random::<f32>()).as_str(),
                ));
                need_to_spawn_enemy_list.push(EnemySpawnNode::new(
                    0,
                    1,
                    spawn_position,
                    format!("rotation={}|", random::<f32>()).as_str(),
                ));
                need_to_spawn_enemy_list.push(EnemySpawnNode::new(
                    0,
                    1,
                    spawn_position,
                    format!("rotation={}|", random::<f32>()).as_str(),
                ));
            }

            {
                let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                play_sound_nodes.insert(String::from("boss_spawn"), (String::from("./resources/sfx/boss_spawn.mp3"), 0.6 ) );
            }

            {
                let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                play_sound_nodes.insert(String::from("boss_enter"), (String::from("./resources/sfx/boss_enter.mp3"), 0.8 ) );
            }

            enemy.spawn_splash(spawn_position, 2.5);
            enemy.spawn_splash(spawn_position, 2.5);
        }
        else if enemy.state == 2  
        {
            if enemy.frame <= 400 && enemy.frame % 15 == 0 
            {
                Enemy::spawn_bullet(
                    enemy.position + Vec2::new(28.0, -4.0),
                    player.unwrap().get_hit_point_position() + Vec2::new(8.0 - random::<f32>() * 16.0, (8.0 - random::<f32>() * 16.0)),
                    1,
                    1.0,
                    4.0,
                    "idle_animation=enemy-bullet-1-idle|firing_animation=enemy-bullet-1-firing|hit_animation=enemy-bullet-1-hit|kill_animation=enemy-bullet-1-kill|scale=1.2|",
                );
                
                Enemy::spawn_bullet(
                    enemy.position + Vec2::new(32.0, -16.0),
                    player.unwrap().get_hit_point_position() + Vec2::new(8.0 - random::<f32>() * 16.0, (8.0 - random::<f32>() * 16.0)),
                    1,
                    1.0,
                    4.0,
                    "idle_animation=enemy-bullet-1-idle|firing_animation=enemy-bullet-1-firing|hit_animation=enemy-bullet-1-hit|kill_animation=enemy-bullet-1-kill|scale=1.2|",
                );
            }
            else if enemy.frame > 450
            {
                enemy.frame = 0;
            }
            
        }
        
    }
}

impl EnemyType for BossEnemyType {
    fn enemy_type_id(&self) -> i32 {
        3
    }

    fn init(&mut self, enemy: &mut Enemy, image_assets: &ImageAssets) {
        enemy.enemy_type = self.enemy_type_id();
        enemy.radius = 24.0;
        enemy.active = true;
        enemy.health = 300;
        enemy.life_time = 100;
        enemy.maximum_tick = 2000;
        enemy.weapon_tick = self.random_weapon_tick();
    
        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("boss_enter"), (String::from("./resources/sfx/boss_enter.mp3"), 0.7 ) );
        }

        match image_assets.get_animation_object("enemy-boss-idle") {
            Some(animation) => {
                enemy.sprite.scale = Vec2::new(1.0, 1.0);
                enemy.sprite.set_loop(true);
                enemy.sprite.play(&animation);
            }
            None => (),
        };

        self.random_target_position(enemy);
    }

    fn update(&self, enemy: &mut Enemy, player: Option<&Player>, image_assets: &ImageAssets) {
        match enemy.tick.checked_add(crate::ONE_FRAME.as_millis()) {
            Some(v) => enemy.tick = v,
            None => enemy.tick = 0,
        };

        match enemy.weapon_tick.checked_sub(crate::ONE_FRAME.as_millis()) {
            Some(v) => enemy.weapon_tick = v,
            None => enemy.weapon_tick = 0,
        };

        enemy.frame += 1;
        self.do_action(enemy, player);

        if enemy.weapon_tick == 0 {
            enemy.weapon_tick = self.random_weapon_tick();

            enemy.frame = 0;
            enemy.state += 1;
            enemy.state = enemy.state % 3;

            
        }

        

        if enemy.tick > enemy.maximum_tick {
            enemy.tick = 0;
            self.random_target_position(enemy);
        }

        let mut actual_position = enemy.position;
        for position in enemy.target_position.iter() {
            actual_position = Vec2::lerp(actual_position, *position, 0.002);
        }
        enemy.position = actual_position;
    }

    fn draw(&self, ctx: &mut Context, image_assets: &ImageAssets, enemy: &mut Enemy) {
        enemy.sprite.draw(ctx, enemy.position, 0.0, image_assets);
    }

    /// This function will called internally in die().
    /// It will decide that what it should do with the bullets on screen when this enemy die
    fn die(&self, enemy: &mut Enemy) {
        enemy.spawn_splash(enemy.position, 3.5);
        enemy.spawn_splash(enemy.position, 3.5);
        enemy.spawn_splash(enemy.position, 3.5);
        enemy.spawn_splash(enemy.position, 3.5);


        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("boss_explode"), (String::from("./resources/sfx/boss_explode.mp3"), 0.8 ) );
        }
    }

    /// Return: 0: not hit, 1: hit weakpoint, -1: hit shield. (No damage)
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
