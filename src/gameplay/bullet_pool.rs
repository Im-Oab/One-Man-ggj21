use std::collections::HashMap;
use std::convert::TryFrom;

use rand::prelude::*;

use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::particle_manager::{ParticleDrawLayer, ParticleSpawnNode};
use crate::image_assets::ImageAssets;
use crate::sprite::Sprite;

pub trait BulletType {
    /// Return type id of the `BulletType`.
    /// This value have to be unique between `BulletType`
    fn bullet_type_id(&self) -> i32;

    /// Setup bullet data.
    /// It will use first split('|') of bullet.extra for `animation_name`.
    /// If no animation. It will set bullet.active to false
    fn setup(&mut self, bullet: &mut Bullet, image_assets: &ImageAssets);

    /// Use for updating bullet position.
    fn update(&self, bullet: &mut Bullet);

    /// Draw bullet on screen. It will be called from inside Bullet.update()
    fn draw(&self, ctx: &mut Context, image_assets: &mut ImageAssets, bullet: &mut Bullet);
}

/// `BulletTypeBank` use for keeping all `BulletType`s that use in the game.
#[derive(Default)]
pub struct BulletTypeBank {
    pub types: HashMap<i32, Box<dyn BulletType + Send + Sync>>,
}

impl BulletTypeBank {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get `BulletType` object by id
    pub fn get(&self, number: i32) -> Option<&(dyn BulletType + Send + Sync)> {
        self.types.get(&number).map(Box::as_ref)
    }

    /// Get mutable `BulletType` object by id
    pub fn get_mut(&mut self, number: i32) -> Option<&mut Box<dyn BulletType + Send + Sync>> {
        self.types.get_mut(&number)
    }

    /// Add new `BulletType` object in the bank.
    pub fn add(&mut self, number: i32, bullet_type: Box<dyn BulletType + Send + Sync>) {
        self.types.insert(number, bullet_type);
    }

    /// Setup default `BulletTypes` in the bank.
    pub fn setup(&mut self, ctx: &mut Context, _image_assets: &ImageAssets) {
        let bullet_type =
            crate::gameplay::bullet_types::constant_velocity::ConstantVelocityBulletType::new(ctx);
        self.add(bullet_type.bullet_type_id(), Box::new(bullet_type));
    }

    /// Clear all `BulletType` objects in the bank
    pub fn clear(&mut self) {
        self.types.clear();
    }
}

/// Owner status of the bullet
#[derive(Copy, Clone)]
pub enum BulletOwner {
    /// Any bullets wihout owner will immediately put back into the pool
    NONE,
    /// Bullet firing from player
    PLAYER(i32),
    /// Bullet firing from enemies
    ENEMY,
}

pub struct Bullet {
    /// active status
    pub active: bool,

    /// reference with BULLET_TYPES
    pub bullet_type: i32,

    /// Which side firing this bullet? PLAYER, ENEMY OR NONE.
    pub owner_type: BulletOwner,

    /// life time decrease over time. The bullet become inactive when it reach 0
    pub life_time: u128,

    /// position of bullet
    pub position: Vec2<f32>,

    /// position from the last update.
    /// We will use this value for interpolate collision detect of the distance between update
    pub previous_position: Vec2<f32>,

    /// rotation value of bullet. (0.0 - 1.0)
    pub rotation: f32,

    /// Speed of the bullet
    pub speed: f32,

    /// use for collision check
    pub radius: f32,

    pub health: i32,
    pub damage: u32,

    pub extra: HashMap<String, String>,
    pub sprite: Sprite,
}

impl Default for Bullet {
    fn default() -> Self {
        Self {
            active: false,
            bullet_type: 0,
            owner_type: BulletOwner::NONE,
            life_time: 0,
            position: Vec2::zero(),
            previous_position: Vec2::zero(),
            rotation: 0.0,
            speed: 0.0,
            radius: 1.0,
            health: 1,
            damage: 1,

            extra: HashMap::new(),
            sprite: Sprite::new(),
        }
    }
}

impl Bullet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.active = false;
        self.bullet_type = 0;
        self.owner_type = BulletOwner::NONE;
        self.life_time = 0;
        self.position = Vec2::zero();
        self.previous_position = Vec2::zero();
        self.rotation = 0.0;
        self.speed = 0.0;
        self.radius = 1.0;
        self.health = 1;
        self.damage = 1;
        self.extra.clear();
        self.sprite.reset();
    }

    pub fn update(&mut self) {
        let mut bullet_type_bank = crate::BULLET_TYPE_BANK.lock().unwrap();
        self.previous_position = self.position;
        if let Some(t) = bullet_type_bank.get_mut(self.bullet_type) {
            t.update(self);
        }

        if !crate::gameplay::utils::is_inside_camera_area(self.position, self.radius) {
            self.life_time = self.life_time.saturating_sub(crate::ONE_FRAME.as_millis());
        }

        if self.life_time == 0 || self.health == 0 {
            self.active = false;
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, image_assets: &mut ImageAssets) {
        let mut bullet_type_bank = crate::BULLET_TYPE_BANK.lock().unwrap();
        if let Some(t) = bullet_type_bank.get_mut(self.bullet_type) {
            t.draw(ctx, image_assets, self);
        }
    }

    pub fn parsing_extra(&mut self, raw_extra: &str) {
        if raw_extra.is_empty() {
            return;
        }

        let split: Vec<&str> = raw_extra.split('|').collect();

        for text in &split {
            if text.is_empty() {
                continue;
            }

            let parameter: Vec<&str> = text.split('=').collect();
            if parameter.len() == 2 {
                self.extra
                    .insert(parameter[0].to_owned(), parameter[1].to_owned());
            } else {
                panic!("Incorrect parameter format: {} ({})", text, raw_extra);
            }
        }
    }

    pub fn spawn_firing_particle(position: Vec2<f32>, raw_extra: &str) {
        Self::spawn_particle(position, ParticleDrawLayer::FiringBullet, raw_extra);
    }

    pub fn spawn_hitting_particle(position: Vec2<f32>, raw_extra: &str) {
        Self::spawn_particle(position, ParticleDrawLayer::BulletHit, raw_extra);
    }

    fn spawn_particle(position: Vec2<f32>, draw_layer: ParticleDrawLayer, raw_extra: &str) {
        let mut particle_spawn_nodes = crate::PARTICLE_SPAWN_NODES.lock().unwrap();
        particle_spawn_nodes.push(ParticleSpawnNode::new(1, position, draw_layer, raw_extra));
    }
}

/// It use for spawing bullet by add it in the queue and every update(). It will get fetched and spawn bullets.
pub struct BulletSpawnNode {
    /// BulletType id of spawning bullet. This will use to reference the type of Bullet.
    pub bullet_type: i32,

    /// Spawning position
    pub position: Vec2<f32>,

    /// Which side firing this bullet? PLAYER, ENEMY OR NONE.
    pub owner_type: BulletOwner,

    /// Rotation of spawning bullet.
    pub rotation: f32,

    /// Speed of the bullet
    pub speed: f32,

    /// use for collision check
    pub radius: f32,

    pub extra: String,
}

/// Use for keeping inactive and active bullets. Also, spawning bullet queue too.
pub struct BulletPool {
    /// Total bullets in the pool. It will use for pre-create bullets in the pool
    total_bullets: i32,

    /// Keep inactive pool in here.
    pool: Vec<Bullet>,
    /// active bullets spawned by players
    pub player_active_bullets: Vec<Bullet>,
    /// active bullets spawned by enemies
    pub enemy_active_bullets: Vec<Bullet>,
    /// spawing bullet queue
    spawned_bullet_list: Vec<Bullet>,
}

impl BulletPool {
    /// Create bullet pool object with pre-created bullet data.
    ///
    /// # Arguments:
    ///
    /// * total - number of bullets that will created with this pool
    ///
    /// # Return:
    ///
    /// * `BulletPool` object
    ///
    #[must_use]
    pub fn new(total: i32) -> Self {
        let mut list: Vec<Bullet> = Vec::new();
        for _index in 0..total {
            list.push(Bullet::new());
        }

        Self {
            total_bullets: total,
            pool: list,
            player_active_bullets: Vec::new(),
            enemy_active_bullets: Vec::new(),
            spawned_bullet_list: Vec::new(),
        }
    }

    /// Clear all active bullets and re-create inacitve bullets in the pool.
    pub fn clear(&mut self) {
        self.pool.clear();
        for _index in 0..self.total_bullets {
            self.pool.push(Bullet::new());
        }

        self.player_active_bullets.clear();
        self.enemy_active_bullets.clear();
        self.spawned_bullet_list.clear();
    }

    /// This function will put bullet into spawning queue after get popped from the pool by weapon.
    pub fn use_bullet(&mut self, bullet: Bullet) {
        self.spawned_bullet_list.push(bullet);
    }

    /// Check spawing bullet queue and put it in active list.
    pub fn spawning_bullets_from_waiting_queue(&mut self) {
        while !self.spawned_bullet_list.is_empty() {
            match self.spawned_bullet_list.pop() {
                Some(b) => self.push_bullet_in_list(b),
                None => break,
            };
        }
    }

    /// Put bullet in active list base on `BulletOwner`. Bullet without owner will put it back into pool. (inactive bullet list)
    fn push_bullet_in_list(&mut self, bullet: Bullet) {
        match bullet.owner_type {
            BulletOwner::NONE => {
                self.push(bullet);
                println!("Try to use bullet without owner. Put it back in the pool");
            }
            BulletOwner::PLAYER(_number) => {
                self.player_active_bullets.push(bullet);
                // println!("Added player bullet: {}", self.player_active_bullets.len());
            }
            BulletOwner::ENEMY => self.enemy_active_bullets.push(bullet),
        };
    }

    /// Push bullet data back to the pool
    pub fn push(&mut self, bullet: Bullet) {
        self.pool.push(bullet);
    }

    /// Pop a bullet from the pool
    pub fn pop(&mut self) -> Option<Bullet> {
        let size = self.pool.len();

        if size < 10 {
            println!("Bullet pool almost exhaust: {}", size);
        }
        self.pool.pop()
    }

    /// Pop multiple bullet datas from the pool. It handle when try to pop bullets more than pool size.
    ///
    /// # Arguments:
    ///
    /// * total - total number for popping bullets
    ///
    /// # Return:
    ///
    /// * Vector of bullet data. Number of data can be equal or less than "total".
    ///
    pub fn pops(&mut self, total: i32) -> Vec<Bullet> {
        let size = self.pool.len();

        if size < 10 {
            println!("Bullet pool almost exhaust: {}", size);
        }

        let final_length = self
            .pool
            .len()
            .saturating_sub(usize::try_from(total).unwrap());

        self.pool.split_off(final_length)
    }

    pub fn spawn_bullets_from_queue(image_assets: &ImageAssets) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        let mut bullet_spawn_nodes = crate::BULLET_SPAWN_NODES.lock().unwrap();
        let mut bullet_type_bank = crate::BULLET_TYPE_BANK.lock().unwrap();

        for node in bullet_spawn_nodes.iter() {
            match bullet_pool.pop() {
                Some(mut bullet) => {
                    bullet.reset();

                    let bullet_type_id = node.bullet_type;

                    bullet.active = true;
                    bullet.bullet_type = node.bullet_type;
                    bullet.owner_type = node.owner_type;
                    bullet.position = node.position;
                    bullet.rotation = node.rotation;
                    bullet.speed = node.speed;
                    bullet.radius = node.radius;
                    bullet.parsing_extra(node.extra.as_str());

                    if let Some(v) = bullet.extra.get("scale") {
                        let scale = v.parse::<f32>().unwrap_or(1.0);
                        bullet.sprite.scale = Vec2::new(scale, scale);
                    };

                    let firing_animation = match bullet.extra.get("firing_animation") {
                        Some(value) => value.as_str(),
                        None => "",
                    };

                    let random_size = 16.0;
                    let random_position = Vec2::new(
                        random_size / 2.0 - random::<f32>() * random_size,
                        random_size / 2.0 - random::<f32>() * random_size,
                    );

                    let rotation = if bullet.rotation < 0.0 {
                        bullet.rotation + 1.0
                    } else {
                        bullet.rotation
                    };

                    let flip_x = if rotation > 0.25 && rotation < 0.75 {
                        "flip_x=1"
                    } else {
                        "flip_x=0"
                    };

                    Bullet::spawn_firing_particle(
                        bullet.position + random_position,
                        format!("idle_animation={}|{}|scale=2.5|", firing_animation, flip_x)
                            .as_str(),
                    );

                    if let Some(t) = bullet_type_bank.get_mut(bullet_type_id) {
                        t.setup(&mut bullet, image_assets);
                    }

                    bullet_pool.use_bullet(bullet);
                }
                None => break,
            };
        }

        bullet_spawn_nodes.clear();

        // Spawn bullets from waiting queue. Any firing bullets always put in to the spawning queue.
        bullet_pool.spawning_bullets_from_waiting_queue();
    }

    pub fn update_active_player_bullets() {
        let mut remove_active_bullet_list = vec![];
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        for (index, bullet) in bullet_pool.player_active_bullets.iter_mut().enumerate() {
            bullet.update();

            if !bullet.active {
                remove_active_bullet_list.push(index);
            }
        }

        for index in remove_active_bullet_list.iter().rev() {
            let inactive_bullet = bullet_pool.player_active_bullets.remove(*index);
            bullet_pool.push(inactive_bullet);
        }

        remove_active_bullet_list.clear();
    }

    pub fn update_active_enemies_bullets() {
        let mut remove_active_bullet_list = vec![];
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        for (index, bullet) in bullet_pool.enemy_active_bullets.iter_mut().enumerate() {
            bullet.update();

            if !bullet.active {
                remove_active_bullet_list.push(index);
            }
        }

        for index in remove_active_bullet_list.iter().rev() {
            let inactive_bullet = bullet_pool.enemy_active_bullets.remove(*index);
            bullet_pool.push(inactive_bullet);
        }

        remove_active_bullet_list.clear();
    }

    pub fn draw_active_player_bullets(ctx: &mut Context, image_assets: &mut ImageAssets) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        for bullet in &mut bullet_pool.player_active_bullets {
            bullet.draw(ctx, image_assets);
        }
    }

    pub fn draw_active_enemies_bullets(ctx: &mut Context, image_assets: &mut ImageAssets) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        for bullet in &mut bullet_pool.enemy_active_bullets {
            bullet.draw(ctx, image_assets);
        }
    }
}
