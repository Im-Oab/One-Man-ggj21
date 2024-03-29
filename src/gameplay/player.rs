use tetra::graphics::{self, Color, GeometryBuilder, Mesh, Rectangle, ShapeStyle};
use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::bullet_pool::{BulletOwner, BulletSpawnNode};
use crate::gameplay::input::{Input, Keyboard};
use crate::image_assets::ImageAssets;
use crate::sprite::Sprite;
use crate::gameplay::enemy_manager::Enemy;

pub enum WeaponType {
    Melee,
    Range,
}

pub enum PlayerState {
    Jump,
    Stand,
    Run,
}

const HIT_POINT_RADIUS: f32 = 4.0;
const MELEE_ATTACK_RADIUS: f32 = 40.0;

pub struct Player {
    player_number: i32,
    health: u32,
    max_health: u32,
    hit_frame: u128,
    is_dead: bool,
    animation_state: PlayerState,
    /// Skin
    pub skin: Sprite,
    slash: Sprite,
    hit_point: Mesh,

    /// Input
    controller: Box<dyn Input>,
    /// Movement
    position: Vec2<f32>,
    direction: i32,
    jump_speed: u128,
    fall_time: u128,
    falling_slow_time: u128,
    dash_speed: i32,

    /// Weapon
    weapon_type: WeaponType,
    melee_attack_time: u128,
    melee_attack_cooldown: u128,
    melee_attack_button_buffer: u128,

    range_attack_time: u128,
    range_attack_cooldown: u128,
    crosshair_position: Vec2<f32>,


}

impl Player {
    pub fn new(ctx: &mut Context, player_number: i32) -> Player {
        let hit_point = GeometryBuilder::new()
            .set_color(Color::RED)
            .circle(ShapeStyle::Fill, Vec2::zero(), HIT_POINT_RADIUS)
            .unwrap()
            .build_mesh(ctx)
            .unwrap();

        let keyboard = Keyboard::new_with_preset_keys();

        Player {
            player_number: player_number,
            health: 20,
            max_health: 20,
            is_dead: false,
            hit_frame: 0,
            animation_state: PlayerState::Stand,

            skin: Sprite::new(),
            slash: Sprite::new(),
            hit_point: hit_point,
            controller: Box::new(keyboard),
            position: Vec2::zero(),
            direction: 1,
            jump_speed: 0,
            fall_time: 0,
            falling_slow_time: 0,
            dash_speed: 0,

            weapon_type: WeaponType::Melee,
            melee_attack_time: 0,
            melee_attack_cooldown: 0,
            melee_attack_button_buffer: 0,

            range_attack_time: 0,
            range_attack_cooldown: 0,
            crosshair_position: Vec2::zero(),
        }
    }

    pub fn setup(&mut self, image_assets: &ImageAssets) {
        match image_assets.get_animation_object("player-stand") {
            Some(animation) => {
                self.skin.play(&animation);
            }
            None => (),
        };
    }

    pub fn update(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        self.controller.update(ctx);
        self.skin.update();
        self.slash.update();
        update_movement(self, image_assets);

        Player::decrease_values_over_time(self);

        if self.health == 0 && self.is_dead == false {
            self.die();

            if self.skin.get_current_animation_name() != "player-die"
            {
                match image_assets.get_animation_object("player-die")
                {
                    Some(animation) => {
                        self.skin.set_loop(false);
                        self.skin.play(&animation);
                    },
                    None => {}
                };
            }
            
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        if self.direction == 1 {
            self.skin.flip_x(false);
        } else {
            self.skin.flip_x(true);
        }

        self.skin
            .draw(ctx, self.get_hit_point_position(), 0.0, image_assets);

        if self.melee_attack_time > 0 {
            self.slash.flip_x(self.direction == -1);
            self.slash
                .draw(ctx, self.get_melee_attack_position(), 0.0, image_assets);
        }

        if self.is_dead == false {
            graphics::draw(ctx, &self.hit_point, self.get_hit_point_position());
        }
    }

    pub fn get_hit(&mut self, damage: u32) {
        if self.hit_frame == 0 && self.melee_attack_time <= 10 && self.dash_speed == 0 {
            match self.health.checked_sub(damage) {
                Some(v) => self.health = v,
                None => self.health = 0,
            }
            self.hit_frame = 90;
            // println!("Hit: {}", self.health);

            Enemy::spawn_random_splash_particle(self.get_hit_point_position(), 1.5);
            Enemy::spawn_random_splash_particle(self.get_hit_point_position(), 1.5);

            {
                let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                play_sound_nodes.insert(String::from("player_hit"), (String::from("./resources/sfx/player_hit.mp3"), 0.8 ) );
            }
        }
    }

    pub fn get_hit_point_position(&self) -> Vec2<f32> {
        self.position + Vec2::new(0.0, -46.0)
    }

    pub fn get_hit_point_radius(&self) -> f32 {
        HIT_POINT_RADIUS
    }

    fn die(&mut self) {
        println!("Player {} die", self.player_number);
        self.is_dead = true;
        self.jump_speed = 30;
        self.fall_time = 0;

        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("player_die"), (String::from("./resources/sfx/player_die.mp3"), 0.8 ) );
        }
    }

    pub fn alive(&self) -> bool {
        self.is_dead == false
    }

    pub fn get_health_percentage(&self) -> f32 {
        self.health as f32 / self.max_health as f32
    }

    fn decrease_values_over_time(player: &mut Player) {
        match player
            .melee_attack_time
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.melee_attack_time = v,
            None => player.melee_attack_time = 0,
        };

        match player
            .range_attack_time
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.range_attack_time = v,
            None => player.range_attack_time = 0,
        };

        match player
            .melee_attack_cooldown
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.melee_attack_cooldown = v,
            None => player.melee_attack_cooldown = 0,
        };

        match player
            .range_attack_cooldown
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.range_attack_cooldown = v,
            None => player.range_attack_cooldown = 0,
        };

        match player
            .melee_attack_button_buffer
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.melee_attack_button_buffer = v,
            None => player.melee_attack_button_buffer = 0,
        };

        match player.hit_frame.checked_sub(crate::ONE_FRAME.as_millis()) {
            Some(v) => player.hit_frame = v,
            None => player.hit_frame = 0,
        };

        match player
            .falling_slow_time
            .checked_sub(crate::ONE_FRAME.as_millis())
        {
            Some(v) => player.falling_slow_time = v,
            None => player.falling_slow_time = 0,
        };
    }

    pub fn is_attacking(&self) -> bool {
        match self.weapon_type {
            WeaponType::Melee => self.melee_attack_time > 0,
            WeaponType::Range => self.range_attack_time > 0,
        }
    }

    pub fn get_crosshair_position(&self) -> &Vec2<f32> {
        &self.crosshair_position
    }
    pub fn get_weapon_type(&self) -> &WeaponType {
        &self.weapon_type
    }

    pub fn get_melee_attack_radius() -> f32 {
        MELEE_ATTACK_RADIUS
    }

    pub fn get_melee_attack_position(&self) -> Vec2<f32> {
        self.position + Vec2::new(10.0 * self.direction as f32, -48.0)
    }

    pub fn melee_attack_damage(&self) -> u32 {
        2
    }

    pub fn melee_attack_hit_enemy(&mut self) {
        self.falling_slow_time = 300;

        if self.melee_attack_cooldown >= 320
        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("melee_hit_target"), (String::from("./resources/sfx/melee_hit_target.mp3"), 0.8 ) );
        }
    }
}

fn apply_gravity(position: &mut Vec2<f32>, falling_slow_time: u128) {
    if falling_slow_time == 0 {
        position.y += crate::GRAVITY;
    } else {
        position.y += crate::GRAVITY * 0.2;
    }

    position.y = position.y.min(crate::GROUND);
}

fn on_the_ground(position: Vec2<f32>) -> bool {
    position.y >= crate::GROUND - 10.0
}

fn update_movement(player: &mut Player, image_assets: &ImageAssets) {
    if player.is_dead == false {
        match player.weapon_type {
            WeaponType::Melee => {
                melee_movement(player, image_assets);
            }
            WeaponType::Range => {
                range_movement(player);
            }
        };
    }

    if on_the_ground(player.position) == false {
        player.animation_state = PlayerState::Jump;
    }

    if player.jump_speed == 0 {
        if player.fall_time > 50 {
            apply_gravity(&mut player.position, player.falling_slow_time);
        }

        player.fall_time += crate::ONE_FRAME.as_millis();
    } else {
        match player.jump_speed.checked_sub(crate::ONE_FRAME.as_millis()) {
            Some(v) => player.jump_speed = v,
            None => player.jump_speed = 0,
        };

        player.position.y -= crate::GRAVITY * 1.2;
    }

    

    if player.is_dead == false
    {
        let animation_name = match player.animation_state {
            PlayerState::Jump => "player-jump",
            PlayerState::Stand => "player-stand",
            PlayerState::Run => "player-run",
        };

        match image_assets.get_animation_object(animation_name) {
            Some(animation) => {
                if player.skin.get_current_animation_name() != animation_name {
                    player.skin.play(&animation);
                }
            }
            None => (),
        };
    }

    

    crate::gameplay::utils::clamp_position_inside_camera_area(&mut player.position);
    crate::gameplay::utils::clamp_position_inside_camera_area(&mut player.crosshair_position);
}

fn melee_movement(player: &mut Player, image_assets: &ImageAssets) {
    let speed = match player.falling_slow_time == 0 {
        true => 6.0,
        false => 2.0,
    };

    player.animation_state = PlayerState::Stand;
    if player.dash_speed == 0
    {
        if player.controller.right() {
            player.position.x += speed;
            if player.is_attacking() == false {
                player.direction = 1;
            }
    
            player.animation_state = PlayerState::Run;
        } else if player.controller.left() {
            player.position.x -= speed;
            if player.is_attacking() == false {
                player.direction = -1;
            }
    
            player.animation_state = PlayerState::Run;
        }
    }
    else
    {
        player.dash_speed -= 1;
        player.dash_speed = player.dash_speed.max(0);
        player.position.x += (player.dash_speed * player.direction) as f32;
        player.animation_state = PlayerState::Run;
    }
    

    if player.controller.up() && on_the_ground(player.position) {
        player.jump_speed = 200;
        player.fall_time = 0;
    }

    if player.controller.attack() {
        player.melee_attack_button_buffer = 80;
    }

    if player.melee_attack_button_buffer > 0
        && player.melee_attack_time == 0
        && player.melee_attack_cooldown == 0
    {
        player.melee_attack_time = 120;
        player.melee_attack_cooldown = 350;

        if player.is_dead == false
        {
            match image_assets.get_animation_object("player-slash") {
                Some(animation) => {
                    player.slash.play(&animation);
                    player.slash.set_loop(false);
                }
                None => (),
            };
        }
        

        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.insert(String::from("melee_hit"), (String::from("./resources/sfx/melee_hit.mp3"), 0.6 ) );
        }
    }

    if player.controller.attack_holding() > 150 && player.dash_speed == 0{
        player.weapon_type = WeaponType::Range;
    }
    else if player.controller.switch() && player.dash_speed == 0
    {
        player.dash_speed = 30;
    }
}

fn range_movement(player: &mut Player) {
    let speed = 8.0;
    player.animation_state = PlayerState::Stand;
    if player.crosshair_position.x < player.position.x {
        player.direction = -1;
    } else {
        player.direction = 1;
    }

    if player.controller.right() {
        player.crosshair_position.x += speed;
    } else if player.controller.left() {
        player.crosshair_position.x -= speed;
    }

    if player.controller.up() {
        player.crosshair_position.y -= speed;
    } else if player.controller.down() {
        player.crosshair_position.y += speed;
    }

    if player.controller.attack_holding() == 0 || player.dash_speed > 0{
        player.weapon_type = WeaponType::Melee;
    }
    else if player.controller.switch() && player.dash_speed == 0
    {
        player.dash_speed = 30;
    }
    

    if player.controller.attack_holding() > 0 && player.range_attack_time == 0 {
        player.range_attack_time = 120;
    }

    if player.range_attack_time > 0 && player.range_attack_cooldown == 0 {
        player.range_attack_cooldown = 60;
        spawn_bullet(
            player.player_number,
            player.get_hit_point_position() + Vec2::new(0.0, -8.0),
            player.crosshair_position,
        );
    }
}

fn spawn_bullet(player_number: i32, from: Vec2<f32>, target: Vec2<f32>) {
    let rotation = (target.y - from.y).atan2(target.x - from.x).to_degrees() / 360.0;

    let mut bullet_spawn_nodes = crate::BULLET_SPAWN_NODES.lock().unwrap();
    bullet_spawn_nodes.push(BulletSpawnNode {
        bullet_type: 1,
        position: from,
        owner_type: BulletOwner::PLAYER(player_number),
        rotation: rotation,
        speed: 10.0,
        radius: 6.0,
        extra: String::from("idle_animation=player-bullet-idle|firing_animation=player-bullet-firing|hit_animation=player-bullet-hit|scale=1.8|"),
    })
}
