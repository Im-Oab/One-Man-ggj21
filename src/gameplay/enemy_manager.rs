use std::collections::HashMap;

use rand::prelude::*;

use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::bullet_pool::{BulletOwner, BulletSpawnNode};
use crate::gameplay::particle_manager::{ParticleDrawLayer, ParticleSpawnNode};
use crate::gameplay::player::Player;

use crate::image_assets::ImageAssets;
use crate::sprite::Sprite;

pub struct EnemyManager {
    active_enemies: Vec<Enemy>,
    inactive_enemies: Vec<Enemy>,
    remove_active_enemy_list: Vec<usize>,
}

impl EnemyManager {
    pub fn new() -> EnemyManager {
        let mut inactive_enemies = vec![];
        for _ in 1..100 {
            inactive_enemies.push(Enemy::new());
        }

        EnemyManager {
            active_enemies: vec![],
            inactive_enemies: inactive_enemies,
            remove_active_enemy_list: vec![],
        }
    }

    pub fn spawn_enemy(
        &mut self,
        enemy_type_number: i32,
        position: Vec2<f32>,
        raw_extra: &str,
        image_assets: &ImageAssets,
    ) -> bool {
        if self.inactive_enemies.len() > 0 {
            let mut enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
            match enemy_type_bank.get_mut(enemy_type_number) {
                Some(enemy_type) => match self.inactive_enemies.pop() {
                    Some(mut v) => {
                        let enemy = &mut v;
                        enemy.reset();
                        enemy.position = position;
                        enemy.parsing_extra(raw_extra);
                        enemy_type.init(enemy, image_assets);
                        self.active_enemies.push(v);

                        return true;
                    }
                    None => println!("No inactive enemy for use"),
                },
                None => println!("No enemy type: {}", enemy_type_number),
            }
        }

        false
    }

    pub fn update_active_enemies(&mut self, player: Option<&Player>, image_assets: &ImageAssets) {
        self.remove_active_enemy_list.clear();

        let mut index = 0;
        for enemy in self.active_enemies.iter_mut() {
            enemy.update(player, image_assets);
            if enemy.active == false {
                self.remove_active_enemy_list.push(index);
            }

            index += 1;
        }

        if self.remove_active_enemy_list.len() > 0 {
            for index in self.remove_active_enemy_list.iter().rev() {
                let removed_enemy = self.active_enemies.remove(*index);
                self.inactive_enemies.push(removed_enemy);
            }
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        for enemy in self.active_enemies.iter_mut() {
            enemy.draw(ctx, image_assets);
        }
    }

    pub fn get_mut_active_enemy(&mut self) -> &mut Vec<Enemy> {
        &mut self.active_enemies
    }

    pub fn has_active_enemy(&self) -> bool {
        self.active_enemies.len() > 0
    }
}

pub struct Enemy {
    pub active: bool,
    pub enemy_type: i32,

    pub health: u32,
    pub max_health: u32,
    pub hit_frame: i32,

    pub position: Vec2<f32>,
    pub rotation: f32,
    pub radius: f32,

    pub frame: u128,
    pub tick: u128,
    pub maximum_tick: u128,
    pub weapon_tick: u128,
    pub life_time: u128,
    pub state: i32,
    pub extra: HashMap<String, String>,
    pub target_position: Vec<Vec2<f32>>,
    pub sprite: Sprite,
}

impl Enemy {
    pub fn new() -> Enemy {
        Enemy {
            active: false,
            enemy_type: 0,
            health: 0,
            max_health: 1,
            hit_frame: 0,
            position: Vec2::zero(),
            rotation: 0.0,
            radius: 0.0,
            frame:0,
            tick: 0,
            maximum_tick: 1,
            weapon_tick: 0,
            life_time: 0,
            state: 0,
            extra: HashMap::new(),
            target_position: vec![],
            sprite: Sprite::new(),
        }
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.enemy_type = 0;
        self.health = 0;
        self.max_health = 1;
        self.position = Vec2::zero();
        self.rotation = 0.0;
        self.radius = 0.0;
        self.frame = 0;
        self.tick = 0;
        self.maximum_tick = 1;
        self.weapon_tick = 0;
        self.life_time = 0;
        self.state = 0;
        self.extra.clear();
        self.target_position.clear();
        self.sprite.reset();
    }

    pub fn update(&mut self, player: Option<&Player>, image_assets: &ImageAssets) {
        self.sprite.update();

        {
            let enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
            match enemy_type_bank.get(self.enemy_type) {
                Some(t) => {
                    t.update(self, player, image_assets);
                }
                None => (),
            };
        }
        

        if self.health == 0 {
            self.die();
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        let enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
        match enemy_type_bank.get(self.enemy_type) {
            Some(t) => {
                t.draw(ctx, image_assets, self);
            }
            None => (),
        };
    }

    pub fn hit_check(&self, position: &Vec2<f32>, radius: f32) -> i32 {
        let enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
        match enemy_type_bank.get(self.enemy_type) {
            Some(t) => {
                let result = t.hit_check(self, position, radius);
                return result;
            }
            None => {
                println!("Not enemy type for hit_check")
            }
        };

        0
    }

    pub fn get_hit(&mut self, hit_position: &Vec2<f32>, damage: u32) {
        if self.hit_frame == 0 {
            self.hit_frame = 8;
        }

        let distance = crate::gameplay::utils::distance_sqr(
            self.position.x as i128,
            self.position.y as i128,
            hit_position.x as i128,
            hit_position.y as i128,
        );

        let hit_position = match distance > (self.radius * self.radius) as i128 {
            true => &self.position,
            false => hit_position,
        };

        // Use hit_position for spawning hitting particle here

        match self.health.checked_sub(damage) {
            Some(v) => self.health = v,
            None => self.health = 0,
        }

        self.spawn_splash(*hit_position, 0.9);
    }

    fn die(&mut self) {
        self.spawn_splash(self.position, 1.6);
        self.active = false;

        let enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
        match enemy_type_bank.get(self.enemy_type) {
            Some(t) => {
                t.die(self);
            }
            None => (),
        };
    }

    pub fn spawn_splash(&self, offset_position: Vec2<f32>, scale_value: f32) {
        Enemy::spawn_random_splash_particle(offset_position, scale_value);
    }

    pub fn spawn_random_splash_particle(offset_position: Vec2<f32>, scale_value: f32) {
        let random_size = 24.0;
        let random_position = Vec2::new(
            random_size / 2.0 - random::<f32>() * random_size,
            random_size / 2.0 - random::<f32>() * random_size,
        );

        let flip_x = if random::<u8>() % 2 == 0 {
            "flip_x=1"
        } else {
            "flip_x=0"
        };

        let splashes = ["splash-1", "splash-2", "splash-3", "splash-4"];

        let name = splashes[random::<usize>() % splashes.len()];

        let position = random_position + offset_position;

        let mut particle_spawn_nodes = crate::PARTICLE_SPAWN_NODES.lock().unwrap();
        particle_spawn_nodes.push(ParticleSpawnNode::new(
            1,
            position,
            ParticleDrawLayer::Explosion,
            format!("idle_animation={}|{}|scale={}|", name, flip_x, scale_value).as_str(),
        ));
    }

    pub fn spawn_bullet(
        from: Vec2<f32>,
        target: Vec2<f32>,
        bullet_type: i32,
        speed: f32,
        radius: f32,
        raw_extra: &str,
    ) {
        let rotation = (target.y - from.y).atan2(target.x - from.x).to_degrees() / 360.0;

        let mut bullet_spawn_nodes = crate::BULLET_SPAWN_NODES.lock().unwrap();
        bullet_spawn_nodes.push(BulletSpawnNode {
            bullet_type: bullet_type,
            position: from,
            owner_type: BulletOwner::ENEMY,
            rotation: rotation,
            speed: speed,
            radius: radius,
            extra: String::from(raw_extra),
        })
    }

    pub fn parsing_extra(&mut self, raw_extra: &str) {
        if raw_extra.len() == 0 {
            return;
        }

        let split: Vec<&str> = raw_extra.split('|').collect();

        for text in split.iter() {
            if text.len() == 0 {
                continue;
            }

            let parameter: Vec<&str> = text.split('=').collect();
            if parameter.len() == 2 {
                self.extra
                    .insert(String::from(parameter[0]), String::from(parameter[1]));
            } else {
                panic!("Incorrect parameter format: {} ({})", text, raw_extra);
            }
        }
    }
}

pub trait EnemyType {
    fn enemy_type_id(&self) -> i32;
    fn init(&mut self, enemy: &mut Enemy, image_assets: &ImageAssets);
    fn update(&self, enemy: &mut Enemy, player: Option<&Player>, image_assets: &ImageAssets);
    fn draw(&self, ctx: &mut Context, image_assets: &ImageAssets, enemy: &mut Enemy);
    fn die(&self, enemy: &mut Enemy);

    /// 0: not hit, 1: hit weakpoint, -1: hit shield. (No damage)
    fn hit_check(&self, enemy: &Enemy, position: &Vec2<f32>, radius: f32) -> i32;
}

pub struct EnemyTypeBank {
    types: HashMap<i32, Box<dyn EnemyType + Send + Sync>>,
}

impl EnemyTypeBank {
    pub fn new() -> EnemyTypeBank {
        EnemyTypeBank {
            types: HashMap::new(),
        }
    }

    pub fn setup(&mut self, image_assets: &ImageAssets, required_list: &Vec<i32>) {
        for enemy_type_number in required_list {
            match enemy_type_number {
                0 => {
                    let enemy_type =
                        crate::gameplay::enemy_types::spawner::SpawnerEnemyType::new(image_assets);
                    self.add(Box::new(enemy_type));
                }
                1 => {
                    let enemy_type =
                        crate::gameplay::enemy_types::flying_pop_corn::FlyingPopCornEnemyType::new(
                            image_assets,
                        );
                    self.add(Box::new(enemy_type));
                }
                2 => {
                    let enemy_type =
                        crate::gameplay::enemy_types::crawling_pop_corn::CrawlingPopCornEnemyType::new(
                            image_assets,
                        );
                    self.add(Box::new(enemy_type));
                },
                3 => {
                    let enemy_type =
                        crate::gameplay::enemy_types::boss::BossEnemyType::new(
                            image_assets,
                        );
                    self.add(Box::new(enemy_type));
                }
                _ => (),
            };
        }
    }

    pub fn add(&mut self, enemy_type: Box<dyn EnemyType + Send + Sync>) {
        self.types.insert(enemy_type.enemy_type_id(), enemy_type);
    }

    pub fn get(&self, number: i32) -> Option<&Box<dyn EnemyType + Send + Sync>> {
        self.types.get(&number)
    }

    pub fn get_mut(&mut self, number: i32) -> Option<&mut Box<dyn EnemyType + Send + Sync>> {
        self.types.get_mut(&number)
    }

    pub fn clear(&mut self) {
        self.types.clear();
    }
}
