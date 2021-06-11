use rand::prelude::*;
use std::collections::VecDeque;

use tetra::graphics::{self, Camera, Color, GeometryBuilder, Rectangle, ShapeStyle};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::Context;
use tetra::audio::{Sound, SoundInstance, SoundState};

use crate::image_assets::{ ImageAssets};
use crate::scene::{Scene, Transition};

use crate::gameplay::bullet_pool::{Bullet, BulletPool};
use crate::gameplay::enemy_manager::{Enemy, EnemyManager};
use crate::gameplay::level::{EnemySpawnNode, Level,  PatternNode};
use crate::gameplay::particle_manager::{ParticleDrawLayer, ParticleManager};
use crate::gameplay::player::{Player, WeaponType};
use crate::gameplay::ui::UI;

enum GamePlayState {
    Loading,
    Preparing,
    Playing,
    LevelCleared,
    GameOver,
}

pub struct GamePlayScene {
    reach_camera_target: bool,
    camera_target_position: Vec2<f32>,
    camera: Camera,
    player: Player,
    image_assets: ImageAssets,
    state: GamePlayState,
    enemy_manager: EnemyManager,
    particle_manager: ParticleManager,
    level: Level,
    waiting_time: u128,

    bgm: Option<SoundInstance>,
    ui: UI,
}

impl GamePlayScene {
    pub fn new(ctx: &mut Context) -> tetra::Result<GamePlayScene> {
        let camera = Camera::new(crate::SCREEN_WIDTH, crate::SCREEN_HEIGHT);

        {
            let mut camera_position = crate::CAMERA_POSITION.lock().unwrap();
            camera_position.x = 0.0;
            camera_position.y = -crate::SCREEN_HEIGHT * 0.3;
        }

        {
            let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
            play_sound_nodes.clear();
        }

        {
            let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
            bullet_pool.clear();
        }
        
        {
            let mut bullet_spawn_nodes = crate::BULLET_SPAWN_NODES.lock().unwrap();
            bullet_spawn_nodes.clear();
        }
        
        {
            let mut bullet_type_bank = crate::BULLET_TYPE_BANK.lock().unwrap();
            bullet_type_bank.clear();
        }
    
        {
            let mut enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
            enemy_type_bank.clear();
        }
        
        {
            let mut enemy_spawn_nodes = crate::ENEMY_SPAWN_NODES.lock().unwrap();
            enemy_spawn_nodes.clear();
        }

        {
            let mut partcie_type_bank = crate::PARTICLE_TYPE_BANK.lock().unwrap();
            partcie_type_bank.clear();
        }
        
        {
            let mut particle_spawn_nodes = crate::PARTICLE_SPAWN_NODES.lock().unwrap();
            particle_spawn_nodes.clear();
        }
  

        let texture_list = vec![];
        let mut image_assets = ImageAssets::new(texture_list);

        let small_blue_circle = GeometryBuilder::new()
            .set_color(Color::BLUE)
            .circle(ShapeStyle::Stroke(8.0), Vec2::zero(), 6.0)
            .unwrap()
            .build_mesh(ctx)
            .unwrap();
        image_assets.add_mesh("small-blue-circle", small_blue_circle);

        let big_red_circle = GeometryBuilder::new()
            .set_color(Color::RED)
            .circle(ShapeStyle::Fill, Vec2::zero(), 8.0)
            .unwrap()
            .build_mesh(ctx)
            .unwrap();
        image_assets.add_mesh("big-red-circle", big_red_circle);

        let small_green_circle = GeometryBuilder::new()
            .set_color(Color::GREEN)
            .circle(ShapeStyle::Stroke(8.0), Vec2::zero(), 6.0)
            .unwrap()
            .build_mesh(ctx)
            .unwrap();
        image_assets.add_mesh("small-green-circle", small_green_circle);

        let simple = GeometryBuilder::new()
            .set_color(Color::BLACK)
            .rectangle(
                ShapeStyle::Stroke(5.0),
                Rectangle::new(-12.0, -48.0, 24.0, 48.0),
            )
            .unwrap()
            .build_mesh(ctx)
            .unwrap();
        image_assets.add_mesh("player-rect", simple);

        setup_textures(&mut image_assets);

        let mut level = Level::new();
        setup_level_for_spawning_enemies(&mut level);
        
       
        Ok(GamePlayScene {
            reach_camera_target: false,
            camera_target_position: Vec2::new(0.0, -crate::SCREEN_HEIGHT * 0.3),
            camera: camera,
            player: Player::new(ctx, 1),
            image_assets: image_assets,
            state: GamePlayState::Loading,
            enemy_manager: EnemyManager::new(),
            particle_manager: ParticleManager::new(),
            level: level,
            waiting_time: 1500,
            bgm: None,
            ui: UI::new(),
        })
    }

    fn play_sound(ctx: &mut Context, path: &str, volume: f32) -> Option<SoundInstance>
    {
        match Sound::new(path)
        {
            Ok(s) => {

                match s.play_with(ctx, volume, 1.0)
                {
                    Ok(instance) => {
                        Some(instance)
                    },
                    Err(e) =>
                    {
                        println!("Play sound error: {} {}", path, e);
                        None
                    }
                }
            }
            , 
            Err(e) =>
            {
                println!("Play sound error: {} {}", path, e);
                None
            }
        }
    }
}

impl Scene for GamePlayScene {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result<Transition> {
        match self.state {
            GamePlayState::Loading => {
                if self.image_assets.is_loading() == true {
                    self.image_assets.loading(ctx);
                } else {
                    self.state = GamePlayState::Preparing;
                }


            }
            GamePlayState::Preparing => {
                // Load all animations in the scene
                setup_animations(&mut self.image_assets);

                self.bgm = GamePlayScene::play_sound(ctx, "./resources/bgm/a.mp3", 0.25);

                {
                    let mut bullet_type_bank = crate::BULLET_TYPE_BANK.lock().unwrap();
                    bullet_type_bank.setup(ctx, &self.image_assets);
                }

                self.player.setup(&mut self.image_assets);

                // Animations have to load before setup UI.
                self.ui.setup(ctx, &self.image_assets);

                // Setup particle manager, particle type bank
                {
                    let mut particle_type_bank = crate::PARTICLE_TYPE_BANK.lock().unwrap();
                    particle_type_bank.setup(&self.image_assets);
                }

                {
                    let mut required_list = vec![];
                    required_list.push(0);
                    required_list.push(1);
                    required_list.push(2);
                    required_list.push(3);

                    let mut enemy_type_bank = crate::ENEMY_TYPE_BANK.lock().unwrap();
                    enemy_type_bank.setup(&self.image_assets, &required_list);
                }

                // Set to camera target node to "start"
                self.level.set_current_node("start");

                {
                    let camera_target_position = crate::CAMERA_POSITION.lock().unwrap();
                    self.camera.position = *camera_target_position;
                }

                self.state = GamePlayState::Playing;
            }
            GamePlayState::Playing => {
                self.update_camera_position();
                
                self.player.update(ctx, &self.image_assets);
                // Update active enemy and remove inactive enemy
                self.enemy_manager
                    .update_active_enemies(Some(&self.player), &self.image_assets);

                self.particle_manager.update(&self.image_assets);

                // Update active bullets and remove inactive bullets
                BulletPool::update_active_enemies_bullets();
                BulletPool::update_active_player_bullets();

                // Check enemy spawn patter for this cameranode
                self.level.update();
                // Spawn enemy that put in the queue by Level

                self.spawn_enemy_in_the_queue();

                // Spawn bullet
                BulletPool::spawn_bullets_from_queue(&self.image_assets);

                // Check GameOver game state
                if self.player.alive() == false {
                    self.state = GamePlayState::GameOver;
                }

                // Do collision detects between objects
                self.call_hit_checks();

                // Check that all enemies killed and no pattern for spawning enemy before fetch the next node.
                self.fetching_next_camera_target();

                {
                    let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                    for (_, (path, volume)) in play_sound_nodes.iter()
                    {
                        GamePlayScene::play_sound(ctx, path, *volume);
                    }
                    play_sound_nodes.clear();
                }

                self.ui.update(ctx);
            }
            GamePlayState::LevelCleared => {
                self.ui.update(ctx);
                self.player.update(ctx, &self.image_assets);
                // Update active enemy and remove inactive enemy
                self.enemy_manager
                    .update_active_enemies(Some(&self.player), &self.image_assets);

                self.particle_manager.update(&self.image_assets);

                // Update active bullets and remove inactive bullets
                BulletPool::update_active_enemies_bullets();
                BulletPool::update_active_player_bullets();
            }
            GamePlayState::GameOver => {
                self.ui.update(ctx);
                self.player.update(ctx, &self.image_assets);
                // Update active enemy and remove inactive enemy
                self.enemy_manager
                    .update_active_enemies(Some(&self.player), &self.image_assets);

                self.particle_manager.update(&self.image_assets);

                // Update active bullets and remove inactive bullets
                BulletPool::update_active_enemies_bullets();
                BulletPool::update_active_player_bullets();

                if input::is_key_released(ctx, Key::Z)
                {
                    if self.bgm.is_some()
                    {
                        self.bgm.as_mut().unwrap().stop();
                    }

                    return Ok(Transition::Replace(Box::new(GamePlayScene::new(ctx)?)));
                }
                
            }
        }

        Ok(Transition::None)
    }

    fn draw(&mut self, ctx: &mut Context) {
        graphics::set_transform_matrix(ctx, self.camera.as_matrix());
        graphics::clear(ctx, Color::rgb8(255, 241, 232));

        let bg_key = String::from("bg");
        for (_, camera_node) in self.level.all_nodes().iter() {
            match self.image_assets.get(&bg_key) {
                Some(bg) => {
                    graphics::draw(
                        ctx,
                        bg,
                        camera_node.position
                            - Vec2::new((bg.width() / 2) as f32, (bg.height() / 2) as f32),
                    );
                }
                None => (),
            };
        }

        match self.state
        {
            GamePlayState::Playing => {
                self.ui.draw_intro(ctx, Vec2::new(crate::SCREEN_WIDTH * -0.2, -crate::SCREEN_HEIGHT * 0.6));
                self.ui.draw_warning(ctx, Vec2::new(crate::SCREEN_WIDTH * 4.0, -crate::SCREEN_HEIGHT * 0.6));
            },
            _ => ()
        };
        

        self.particle_manager
            .draw(ParticleDrawLayer::Bottomest, ctx, &self.image_assets);

        self.enemy_manager.draw(ctx, &self.image_assets);

        self.particle_manager
            .draw(ParticleDrawLayer::Explosion, ctx, &self.image_assets);

        self.player.draw(ctx, &self.image_assets);

        self.particle_manager
            .draw(ParticleDrawLayer::BulletHit, ctx, &self.image_assets);

        self.particle_manager
            .draw(ParticleDrawLayer::FiringBullet, ctx, &self.image_assets);

        BulletPool::draw_active_player_bullets(ctx, &mut self.image_assets);
        BulletPool::draw_active_enemies_bullets(ctx, &mut self.image_assets);

        self.particle_manager
            .draw(ParticleDrawLayer::Topest, ctx, &self.image_assets);
        self.ui.draw_crosshair(
            ctx,
            &mut self.image_assets,
            self.player.get_weapon_type(),
            self.player.get_crosshair_position(),
        );
        graphics::reset_transform_matrix(ctx);

        self.ui.draw_energy_bar(
            ctx,
            &mut self.image_assets,
            self.player.get_health_percentage(),
        );

        self.ui
            .draw_weapon(ctx, &mut self.image_assets, self.player.get_weapon_type());

        match self.state
        {
            GamePlayState::GameOver => {
                self.ui.draw_game_over(ctx);
            },
            GamePlayState::LevelCleared => {
                self.ui.draw_level_cleared(ctx);
            }
            _ => {
                
            }
        };
    }
}

impl GamePlayScene {
    /// interporate between current camera position and latest target position. (from CAMERA_POSITION)
    fn update_camera_position(&mut self) {
        self.camera.update();

        if self.reach_camera_target == false {
            let move_speed = 2.0;
            let camera_target_position = self.camera_target_position;

            if self.camera.position.x < camera_target_position.x {
                self.camera.position.x += move_speed
            } else if self.camera.position.x > camera_target_position.x {
                self.camera.position.x -= move_speed;
            }

            if self.camera.position.y < camera_target_position.y {
                self.camera.position.y += move_speed;
            } else if self.camera.position.y > camera_target_position.y {
                self.camera.position.y -= move_speed;
            }

            let distance_sqr =
                Vec2::distance_squared(self.camera.position, self.camera_target_position);
            if distance_sqr <= (move_speed * move_speed) * 1.0 {
                self.reach_camera_target = true;
                self.camera.position = self.camera_target_position;
                println!("camera reach target");
            }

            let mut camera_position = crate::CAMERA_POSITION.lock().unwrap();
            *camera_position = self.camera.position;
        }
    }

    fn fetching_next_camera_target(&mut self) {
        // Go next node if possible
        if self.reach_camera_target == true {
            if self.enemy_manager.has_active_enemy() == false
                && self.level.is_spawn_queue_empty() == true
            {
                if self.waiting_time == 0 {
                    match self.level.get_next_node() {
                        Some(node) => {
                            self.camera_target_position = node.position;
                            self.reach_camera_target = false;
                            let next_node_name = String::from(node.name.as_str());
                            self.waiting_time = node.waiting_time;

                            self.level.set_current_node(&next_node_name);
                        }
                        None => {
                            self.level.set_current_node("");
                            self.waiting_time = 1500;
                        }
                    };
                } else {
                    match self.waiting_time.checked_sub(crate::ONE_FRAME.as_millis()) {
                        Some(v) => self.waiting_time = v,
                        None => self.waiting_time = 0,
                    };
                }
            }
        }
    }

    fn update_hit_check_between_player_melee_attack_with_enemies_bullets(player: &mut Player) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();
        match player.get_weapon_type() {
            WeaponType::Range => {
                return;
            }
            WeaponType::Melee => {
                if player.is_attacking() == false {
                    return;
                }
            }
        }

        for bullet in bullet_pool.enemy_active_bullets.iter_mut() {
            if bullet.active {
                if crate::gameplay::utils::is_inside_camera_area(&bullet.position, bullet.radius)
                    == false
                {
                    continue;
                }

                let total_radius = bullet.radius + Player::get_melee_attack_radius();
                let distance =
                    Vec2::distance_squared(player.get_melee_attack_position(), bullet.position);
                if distance < total_radius * total_radius {
                    bullet.health = 0;

                    match bullet.extra.get("kill_animation") {
                        Some(name) => {
                            let random_size = 8.0;
                            let random_position = Vec2::new(
                                random_size / 2.0 - random::<f32>() * random_size,
                                random_size / 2.0 - random::<f32>() * random_size,
                            );

                            Bullet::spawn_hitting_particle(
                                bullet.position + random_position,
                                format!("idle_animation={}|flip_x=0|", name).as_str(),
                            );
                        }
                        None => (),
                    };
                }
            }
        }
    }

    fn update_hit_check_between_player_melee_attack_with_enemies(
        player: &mut Player,
        active_enemies: &mut Vec<Enemy>,
    ) {
        match player.get_weapon_type() {
            WeaponType::Range => {
                return;
            }
            WeaponType::Melee => {
                if player.is_attacking() == false {
                    return;
                }
            }
        }

        for enemy in active_enemies.iter_mut() {
            if enemy.active {
                if crate::gameplay::utils::is_inside_camera_area(&enemy.position, enemy.radius)
                    == false
                {
                    continue;
                }

                let total_radius = enemy.radius + Player::get_melee_attack_radius();
                let distance = crate::gameplay::utils::distance_sqr(
                    player.get_melee_attack_position().x as i128,
                    player.get_melee_attack_position().y as i128,
                    enemy.position.x as i128,
                    enemy.position.y as i128,
                ) as f32;

                if distance < total_radius * total_radius {
                    let hit_position = enemy.position;
                    enemy.get_hit(&hit_position, player.melee_attack_damage());
                    player.melee_attack_hit_enemy();
                }
            }
        }
    }

    fn update_hit_check_between_player_bullet_and_enemies(active_enemies: &mut Vec<Enemy>) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();

        for bullet in bullet_pool.player_active_bullets.iter_mut() {
            for enemy in active_enemies.iter_mut() {
                if bullet.active == true && enemy.active {
                    if crate::gameplay::utils::is_inside_camera_area(
                        &bullet.position,
                        bullet.radius,
                    ) == false
                    {
                        break;
                    }

                    if crate::gameplay::utils::is_inside_camera_area(&enemy.position, enemy.radius)
                        == false
                    {
                        continue;
                    }

                    let mut break_loop = false;
                    {
                        let checking_position = bullet.position;

                        let result = enemy.hit_check(&checking_position, bullet.radius);

                        if result == 1 || result == -1 {
                            bullet.health -= 1;

                            let bullet_rotation = if bullet.rotation < 0.0 {
                                bullet.rotation + 1.0
                            } else {
                                bullet.rotation
                            };

                            match bullet.extra.get("hit_animation") {
                                Some(name) => {
                                    let random_size = 8.0;
                                    let random_position = Vec2::new(
                                        random_size / 2.0 - random::<f32>() * random_size,
                                        random_size / 2.0 - random::<f32>() * random_size,
                                    );

                                    let flip_x = if bullet_rotation > 0.25 && bullet_rotation < 0.75
                                    {
                                        "flip_x=1"
                                    } else {
                                        "flip_x=0"
                                    };

                                    Bullet::spawn_hitting_particle(
                                        bullet.position + random_position,
                                        format!("idle_animation={}|{}|", name, flip_x).as_str(),
                                    );
                                }
                                None => (),
                            };

                            if result == 1 {
                                enemy.get_hit(&checking_position, bullet.damage);

                                {
                                    let sfx_list = ["bullet_hit_1", "bullet_hit_2", "bullet_hit_3"];

                                    let name = sfx_list[random::<usize>() % sfx_list.len()];
                                    let mut play_sound_nodes = crate::PLAY_SOUND_NODES.lock().unwrap();
                                    play_sound_nodes.insert(String::from(name), (format!("./resources/sfx/{}.mp3", name), 0.15 ) );
                                }
                            }

                            if enemy.health <= 0 {
                                enemy.active = false;
                            }

                            if bullet.health <= 0 {
                                bullet.active = false;
                                break_loop = true;
                            }

                            break;
                        }
                    }

                    if break_loop {
                        break;
                    }
                }
            }
        }
    }

    fn update_hit_check_between_player_and_enemies(
        player: &mut Player,
        active_enemies: &mut Vec<Enemy>,
    ) {
        let player_hit_point_position = player.get_hit_point_position();
        let player_hit_point_radius = player.get_hit_point_radius();

        for enemy in active_enemies.iter_mut() {
            if enemy.active {
                if crate::gameplay::utils::is_inside_camera_area(&enemy.position, enemy.radius)
                    == false
                {
                    continue;
                }
            }

            if enemy.hit_check(&player_hit_point_position, player_hit_point_radius) != 0 {
                player.get_hit(2);
            }
        }
    }

    fn update_hit_check_between_player_and_enemies_bullets(player: &mut Player) {
        let mut bullet_pool = crate::BULLET_POOL.lock().unwrap();

        let player_hit_point_position = player.get_hit_point_position();
        let player_hit_point_radius = player.get_hit_point_radius();

        for bullet in bullet_pool.enemy_active_bullets.iter_mut() {
            if bullet.active {
                if crate::gameplay::utils::is_inside_camera_area(&bullet.position, bullet.radius)
                    == false
                {
                    continue;
                }

                let mut break_loop = false;
                {
                    let checking_position = bullet.position;
                    
                    let distance = crate::gameplay::utils::distance_sqr(
                        player_hit_point_position.x as i128,
                        player_hit_point_position.y as i128,
                        checking_position.x as i128,
                        checking_position.y as i128,
                    );
                    let total_radius = player_hit_point_radius + bullet.radius;

                    if distance <= (total_radius * total_radius) as i128 {
                        bullet.health -= 1;

                        let bullet_rotation = if bullet.rotation < 0.0 {
                            bullet.rotation + 1.0
                        } else {
                            bullet.rotation
                        };

                        match bullet.extra.get("hit_animation") {
                            Some(name) => {
                                let random_size = 8.0;
                                let random_position = Vec2::new(
                                    random_size / 2.0 - random::<f32>() * random_size,
                                    random_size / 2.0 - random::<f32>() * random_size,
                                );

                                let flip_x = if bullet_rotation > 0.25 && bullet_rotation < 0.75 {
                                    "flip_x=1"
                                } else {
                                    "flip_x=0"
                                };

                                Bullet::spawn_hitting_particle(
                                    bullet.position + random_position,
                                    format!("idle_animation={}|{}|", name, flip_x).as_str(),
                                );
                            }
                            None => (),
                        };

                        player.get_hit(1);

                        if bullet.health <= 0 {
                            bullet.active = false;
                            break_loop = true;
                        }

                        break;
                    }
                }

                if break_loop {
                    break;
                }
            }
        }
    }
}

impl GamePlayScene {
    fn spawn_enemy_in_the_queue(&mut self) {
        // check spawn queue in Level
        let mut need_to_spawn_enemy_list = crate::ENEMY_SPAWN_NODES.lock().unwrap();
        if need_to_spawn_enemy_list.len() > 0 {
            for spawn_node in need_to_spawn_enemy_list.iter() {
                let world_position = spawn_node.position;

                // Add enemy into active list
                self.enemy_manager.spawn_enemy(
                    spawn_node.enemy_type,
                    world_position,
                    spawn_node.extra.as_str(),
                    &self.image_assets,
                );
            }
            need_to_spawn_enemy_list.clear();
        }
        // Check level cleared game state.
        else if need_to_spawn_enemy_list.len() == 0
            && self.level.is_spawn_queue_empty() == true
            && self.enemy_manager.has_active_enemy() == false
            && self.level.get_current_node().is_none() == true
        {
            self.state = GamePlayState::LevelCleared;
        }
    }

    fn call_hit_checks(&mut self) {
        GamePlayScene::update_hit_check_between_player_bullet_and_enemies(
            self.enemy_manager.get_mut_active_enemy(),
        );
        GamePlayScene::update_hit_check_between_player_and_enemies(
            &mut self.player,
            self.enemy_manager.get_mut_active_enemy(),
        );
        GamePlayScene::update_hit_check_between_player_and_enemies_bullets(&mut self.player);

        GamePlayScene::update_hit_check_between_player_melee_attack_with_enemies_bullets(
            &mut self.player,
        );
        GamePlayScene::update_hit_check_between_player_melee_attack_with_enemies(
            &mut self.player,
            self.enemy_manager.get_mut_active_enemy(),
        );
    }
}

fn setup_level_for_spawning_enemies(level: &mut Level) {
    setup_camera_target_nodes(level);
    setup_level_patterns(level);
}

fn setup_camera_target_nodes(level: &mut Level) {
    let mut pos_x = 0.0;
    {
        let mut spawn_pattern = VecDeque::new();

        level.add_camera_target_node(
            "start",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            2000,
            "01",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();
        spawn_pattern.push_back(PatternNode {
            delay: 500,
            pattern: String::from("01"),
        });

        level.add_camera_target_node(
            "01",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            200,
            "02",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();
        spawn_pattern.push_back(PatternNode {
            delay: 500,
            pattern: String::from("02"),
        });

        level.add_camera_target_node(
            "02",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            300,
            "03",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();
        spawn_pattern.push_back(PatternNode {
            delay: 500,
            pattern: String::from("03"),
        });
        level.add_camera_target_node(
            "03",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            300,
            "boss",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();

        level.add_camera_target_node(
            "04",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            0,
            "05",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();

        level.add_camera_target_node(
            "05",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            0,
            "boss",
            spawn_pattern,
        );
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        let mut spawn_pattern = VecDeque::new();
        spawn_pattern.push_back(PatternNode {
            delay: 500,
            pattern: String::from("boss"),
        });

        level.add_camera_target_node(
            "boss",
            Vec2::new(pos_x, -crate::SCREEN_HEIGHT * 0.3),
            300,
            "",
            spawn_pattern,
        );
    }
}

fn setup_level_patterns(level: &mut Level) {
    let mut pos_x = 0.0;

    pos_x += crate::SCREEN_WIDTH;
    {
        {
            let mut pattern = VecDeque::new();
            pattern.push_back(EnemySpawnNode::new(
                0,
                0,
                Vec2::new( pos_x , crate::GROUND - 52.0),
                "spawn_time=3500|spawn_interval=80|spawn_queue=1111111|idle_animation=enemy-spawner-1-idle|spawning_animation=enemy-spawner-1-spawning|scale=1.2|flip_x=1|",
            ));

            level.add_pattern("01", pattern);
        }
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        {
            let mut pattern = VecDeque::new();

            pattern.push_back(EnemySpawnNode::new(
                0,
                0,
                Vec2::new( pos_x - crate::SCREEN_WIDTH * 0.3 , crate::GROUND - 16.0),
                "spawn_time=4000|spawn_interval=100|spawn_queue=111111|idle_animation=enemy-spawner-2-idle|spawning_animation=enemy-spawner-2-spawning|scale=1.4|flip_x=0|",
            ));

            pattern.push_back(EnemySpawnNode::new(
                0,
                0,
                Vec2::new( pos_x + crate::SCREEN_WIDTH * 0.3 , crate::GROUND - 16.0),
                "spawn_time=7000|spawn_interval=100|spawn_queue=111111|idle_animation=enemy-spawner-2-idle|spawning_animation=enemy-spawner-2-spawning|scale=1.4|flip_x=1|",
            ));

            pattern.push_back(EnemySpawnNode::new(
                3500,
                2,
                Vec2::new(pos_x + crate::SCREEN_WIDTH * 0.5, crate::GROUND - 52.0),
                "rotation=0.35|",
            ));
            level.add_pattern("02", pattern);
        }
    }

    pos_x += crate::SCREEN_WIDTH;
    {
        {
            let mut pattern = VecDeque::new();

            pattern.push_back(EnemySpawnNode::new(
                0,
                0,
                Vec2::new( pos_x + 120.0 , crate::GROUND - 52.0),
                "spawn_time=3500|spawn_interval=120|spawn_queue=111111111|idle_animation=enemy-spawner-1-idle|spawning_animation=enemy-spawner-1-spawning|scale=1.2|flip_x=1|",
            ));

            pattern.push_back(EnemySpawnNode::new(
                3500,
                2,
                Vec2::new(pos_x + crate::SCREEN_WIDTH * 0.6, crate::GROUND - 52.0),
                "rotation=0.4|",
            ));

            pattern.push_back(EnemySpawnNode::new(
                3500,
                2,
                Vec2::new(pos_x - crate::SCREEN_WIDTH * 0.5, crate::GROUND - 52.0),
                "rotation=0.2|",
            ));

            

            level.add_pattern("03", pattern);
        }
    }

    pos_x += crate::SCREEN_WIDTH;
    pos_x += crate::SCREEN_WIDTH;
    pos_x += crate::SCREEN_WIDTH;
    {
        {
            let mut pattern = VecDeque::new();

            pattern.push_back(EnemySpawnNode::new(
                4500,
                3,
                Vec2::new( pos_x + crate::SCREEN_WIDTH , crate::GROUND - 120.0),
                "",
            ));

            level.add_pattern("boss", pattern);
        }
    }
}

fn setup_textures(image_assets: &mut ImageAssets) {
    image_assets.add_content("bg", "./resources/bg.png");

    for index in 1..5 {
        image_assets.add_content(
            format!("ui-bar-{}", index).as_str(),
            format!("./resources/ui/bar/{}.png", index).as_str(),
        );
    }
    image_assets.add_content("ui-bar-bg", "./resources/ui/bar/bg.png");
    image_assets.add_content("ui-bar-fg", "./resources/ui/bar/fg.png");

    for index in 1..9 {
        image_assets.add_content(
            format!("ui-circle-{}", index).as_str(),
            format!("./resources/ui/circle/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("ui-crosshair-{}", index).as_str(),
            format!("./resources/ui/crosshair/{}.png", index).as_str(),
        );
    }
    image_assets.add_content(
        "ui-crosshair-inactive",
        "./resources/ui/crosshair/inactive.png",
    );

    image_assets.add_content("ui-weapon-melee", "./resources/ui/weapons/melee.png");
    image_assets.add_content("ui-weapon-range", "./resources/ui/weapons/range.png");

    for index in 1..5 {
        image_assets.add_content(
            format!("ui-z-{}", index).as_str(),
            format!("./resources/ui/z/{}.png", index).as_str(),
        );
    }

    for index in 1..12 {
        image_assets.add_content(
            format!("player-run-{}", index).as_str(),
            format!("./resources/player/run/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("player-stand-{}", index).as_str(),
            format!("./resources/player/stand/{}.png", index).as_str(),
        );
    }

    for index in 1..6 {
        image_assets.add_content(
            format!("player-jump-{}", index).as_str(),
            format!("./resources/player/jump/{}.png", index).as_str(),
        );
    }

    for index in 1..10 {
        image_assets.add_content(
            format!("player-die-{}", index).as_str(),
            format!("./resources/player/die/{}.png", index).as_str(),
        );
    }

    for index in 1..7 {
        image_assets.add_content(
            format!("player-slash-{}", index).as_str(),
            format!("./resources/player/slash/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("player-bullet-idle-{}", index).as_str(),
            format!("./resources/player/bullet/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..4 {
        image_assets.add_content(
            format!("player-bullet-firing-{}", index).as_str(),
            format!("./resources/player/bullet/firing/{}.png", index).as_str(),
        );
    }

    for index in 1..4 {
        image_assets.add_content(
            format!("player-bullet-hit-{}", index).as_str(),
            format!("./resources/player/bullet/hit/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("enemy-bullet-1-idle-{}", index).as_str(),
            format!("./resources/enemies/bullet-1/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("enemy-bullet-1-hit-{}", index).as_str(),
            format!("./resources/enemies/bullet-1/hit/{}.png", index).as_str(),
        );
    }

    for index in 1..3 {
        image_assets.add_content(
            format!("enemy-bullet-1-firing-{}", index).as_str(),
            format!("./resources/enemies/bullet-1/firing/{}.png", index).as_str(),
        );
    }

    for index in 1..4 {
        image_assets.add_content(
            format!("enemy-bullet-1-kill-{}", index).as_str(),
            format!("./resources/enemies/bullet-1/kill/{}.png", index).as_str(),
        );
    }

    for index in 1..9 {
        image_assets.add_content(
            format!("enemy-flying-idle-{}", index).as_str(),
            format!("./resources/enemies/flying/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..11 {
        image_assets.add_content(
            format!("enemy-flying-spawn-{}", index).as_str(),
            format!("./resources/enemies/flying/spawn/{}.png", index).as_str(),
        );
    }

    for index in 1..7 {
        image_assets.add_content(
            format!("splash-1-{}", index).as_str(),
            format!("./resources/splashes/splash-1/{}.png", index).as_str(),
        );
    }

    for index in 1..7 {
        image_assets.add_content(
            format!("splash-2-{}", index).as_str(),
            format!("./resources/splashes/splash-2/{}.png", index).as_str(),
        );
    }

    for index in 1..8 {
        image_assets.add_content(
            format!("splash-3-{}", index).as_str(),
            format!("./resources/splashes/splash-3/{}.png", index).as_str(),
        );
    }

    for index in 1..10 {
        image_assets.add_content(
            format!("splash-4-{}", index).as_str(),
            format!("./resources/splashes/splash-4/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("enemy-spawner-1-spawning-{}", index).as_str(),
            format!("./resources/enemies/spawner-1/spawning/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("enemy-spawner-1-idle-{}", index).as_str(),
            format!("./resources/enemies/spawner-1/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..5 {
        image_assets.add_content(
            format!("enemy-spawner-2-spawning-{}", index).as_str(),
            format!("./resources/enemies/spawner-2/spawning/{}.png", index).as_str(),
        );
    }

    for index in 1..2 {
        image_assets.add_content(
            format!("enemy-spawner-2-idle-{}", index).as_str(),
            format!("./resources/enemies/spawner-2/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..8 {
        image_assets.add_content(
            format!("enemy-crawler-air-{}", index).as_str(),
            format!("./resources/enemies/crawler/air/{}.png", index).as_str(),
        );
    }

    for index in 1..10 {
        image_assets.add_content(
            format!("enemy-crawler-idle-{}", index).as_str(),
            format!("./resources/enemies/crawler/idle/{}.png", index).as_str(),
        );
    }

    for index in 1..10 {
        image_assets.add_content(
            format!("enemy-boss-idle-{}", index).as_str(),
            format!("./resources/enemies/boss/idle/{}.png", index).as_str(),
        );
    }
}

fn setup_animations(image_assets: &mut ImageAssets) {
    let mut animations = vec![];

    animations.push((
        "enemy-boss-idle",
        vec![
            "enemy-boss-idle-1".to_string(),
            "enemy-boss-idle-2".to_string(),
            "enemy-boss-idle-3".to_string(),
            "enemy-boss-idle-4".to_string(),
            "enemy-boss-idle-5".to_string(),
            "enemy-boss-idle-6".to_string(),
            "enemy-boss-idle-7".to_string(),
            "enemy-boss-idle-8".to_string(),
            "enemy-boss-idle-9".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-crawler-idle",
        vec![
            "enemy-crawler-idle-1".to_string(),
            "enemy-crawler-idle-2".to_string(),
            "enemy-crawler-idle-3".to_string(),
            "enemy-crawler-idle-4".to_string(),
            "enemy-crawler-idle-5".to_string(),
            "enemy-crawler-idle-6".to_string(),
            "enemy-crawler-idle-7".to_string(),
            "enemy-crawler-idle-8".to_string(),
            "enemy-crawler-idle-9".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-crawler-air",
        vec![
            "enemy-crawler-air-1".to_string(),
            "enemy-crawler-air-2".to_string(),
            "enemy-crawler-air-3".to_string(),
            "enemy-crawler-air-4".to_string(),
            "enemy-crawler-air-5".to_string(),
            "enemy-crawler-air-6".to_string(),
            "enemy-crawler-air-7".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-spawner-2-idle",
        vec!["enemy-spawner-2-idle-1".to_string()],
        1,
    ));

    animations.push((
        "enemy-spawner-2-spawning",
        vec![
            "enemy-spawner-2-spawning-1".to_string(),
            "enemy-spawner-2-spawning-2".to_string(),
            "enemy-spawner-2-spawning-3".to_string(),
            "enemy-spawner-2-spawning-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-spawner-1-idle",
        vec![
            "enemy-spawner-1-idle-1".to_string(),
            "enemy-spawner-1-idle-2".to_string(),
            "enemy-spawner-1-idle-3".to_string(),
            "enemy-spawner-1-idle-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-spawner-1-spawning",
        vec![
            "enemy-spawner-1-spawning-1".to_string(),
            "enemy-spawner-1-spawning-2".to_string(),
            "enemy-spawner-1-spawning-3".to_string(),
            "enemy-spawner-1-spawning-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-flying-idle",
        vec![
            "enemy-flying-idle-1".to_string(),
            "enemy-flying-idle-2".to_string(),
            "enemy-flying-idle-3".to_string(),
            "enemy-flying-idle-4".to_string(),
            "enemy-flying-idle-5".to_string(),
            "enemy-flying-idle-6".to_string(),
            "enemy-flying-idle-7".to_string(),
            "enemy-flying-idle-8".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-flying-spawn",
        vec![
            "enemy-flying-spawn-1".to_string(),
            "enemy-flying-spawn-2".to_string(),
            "enemy-flying-spawn-3".to_string(),
            "enemy-flying-spawn-4".to_string(),
            "enemy-flying-spawn-5".to_string(),
            "enemy-flying-spawn-6".to_string(),
            "enemy-flying-spawn-7".to_string(),
            "enemy-flying-spawn-8".to_string(),
            "enemy-flying-spawn-9".to_string(),
            "enemy-flying-spawn-10".to_string(),
        ],
        12,
    ));

    animations.push((
        "splash-1",
        vec![
            "splash-1-1".to_string(),
            "splash-1-2".to_string(),
            "splash-1-3".to_string(),
            "splash-1-4".to_string(),
            "splash-1-5".to_string(),
            "splash-1-6".to_string(),
        ],
        18,
    ));

    animations.push((
        "splash-2",
        vec![
            "splash-2-1".to_string(),
            "splash-2-2".to_string(),
            "splash-2-3".to_string(),
            "splash-2-4".to_string(),
            "splash-2-5".to_string(),
            "splash-2-6".to_string(),
        ],
        18,
    ));

    animations.push((
        "splash-3",
        vec![
            "splash-3-1".to_string(),
            "splash-3-2".to_string(),
            "splash-3-3".to_string(),
            "splash-3-4".to_string(),
            "splash-3-5".to_string(),
            "splash-3-6".to_string(),
            "splash-3-7".to_string(),
        ],
        18,
    ));

    animations.push((
        "splash-4",
        vec![
            "splash-4-1".to_string(),
            "splash-4-2".to_string(),
            "splash-4-3".to_string(),
            "splash-4-4".to_string(),
            "splash-4-5".to_string(),
            "splash-4-6".to_string(),
            "splash-4-7".to_string(),
            "splash-4-8".to_string(),
            "splash-4-9".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-bullet-1-idle",
        vec![
            "enemy-bullet-1-idle-1".to_string(),
            "enemy-bullet-1-idle-2".to_string(),
            "enemy-bullet-1-idle-3".to_string(),
            "enemy-bullet-1-idle-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-bullet-1-hit",
        vec![
            "enemy-bullet-1-hit-1".to_string(),
            "enemy-bullet-1-hit-2".to_string(),
            "enemy-bullet-1-hit-3".to_string(),
            "enemy-bullet-1-hit-4".to_string(),
        ],
        14,
    ));

    animations.push((
        "enemy-bullet-1-firing",
        vec![
            "enemy-bullet-1-firing-1".to_string(),
            "enemy-bullet-1-firing-2".to_string(),
        ],
        18,
    ));

    animations.push((
        "enemy-bullet-1-kill",
        vec![
            "enemy-bullet-1-kill-1".to_string(),
            "enemy-bullet-1-kill-2".to_string(),
            "enemy-bullet-1-kill-3".to_string(),
        ],
        14,
    ));

    animations.push((
        "player-run",
        vec![
            "player-run-1".to_string(),
            "player-run-2".to_string(),
            "player-run-3".to_string(),
            "player-run-4".to_string(),
            "player-run-5".to_string(),
            "player-run-6".to_string(),
            "player-run-7".to_string(),
            "player-run-8".to_string(),
            "player-run-9".to_string(),
            "player-run-10".to_string(),
            "player-run-11".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-slash",
        vec![
            "player-slash-1".to_string(),
            "player-slash-2".to_string(),
            "player-slash-2".to_string(),
            "player-slash-3".to_string(),
            "player-slash-4".to_string(),
            "player-slash-5".to_string(),
            "player-slash-6".to_string(),
        ],
        14,
    ));

    animations.push((
        "player-stand",
        vec![
            "player-stand-1".to_string(),
            "player-stand-2".to_string(),
            "player-stand-3".to_string(),
            "player-stand-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-jump",
        vec![
            "player-jump-1".to_string(),
            "player-jump-2".to_string(),
            "player-jump-3".to_string(),
            "player-jump-4".to_string(),
            "player-jump-5".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-die",
        vec![
            "player-die-1".to_string(),
            "player-die-2".to_string(),
            "player-die-3".to_string(),
            "player-die-4".to_string(),
            "player-die-5".to_string(),
            "player-die-6".to_string(),
            "player-die-7".to_string(),
            "player-die-8".to_string(),
            "player-die-9".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-bullet-idle",
        vec![
            "player-bullet-idle-1".to_string(),
            "player-bullet-idle-2".to_string(),
            "player-bullet-idle-3".to_string(),
            "player-bullet-idle-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-bullet-hit",
        vec![
            "player-bullet-hit-1".to_string(),
            "player-bullet-hit-2".to_string(),
            "player-bullet-hit-3".to_string(),
        ],
        18,
    ));

    animations.push((
        "player-bullet-firing",
        vec![
            "player-bullet-firing-1".to_string(),
            "player-bullet-firing-2".to_string(),
            "player-bullet-firing-3".to_string(),
        ],
        18,
    ));

    animations.push((
        "ui-circle",
        vec![
            "ui-circle-1".to_string(),
            "ui-circle-2".to_string(),
            "ui-circle-3".to_string(),
            "ui-circle-4".to_string(),
            "ui-circle-5".to_string(),
            "ui-circle-6".to_string(),
            "ui-circle-7".to_string(),
            "ui-circle-8".to_string(),
        ],
        24,
    ));

    animations.push((
        "ui-z",
        vec![
            "ui-z-1".to_string(),
            "ui-z-2".to_string(),
            "ui-z-3".to_string(),
            "ui-z-4".to_string(),
        ],
        24,
    ));

    animations.push(("ui-weapon-melee", vec!["ui-weapon-melee".to_string()], 1));

    animations.push(("ui-weapon-range", vec!["ui-weapon-range".to_string()], 1));

    animations.push((
        "ui-bar",
        vec![
            "ui-bar-1".to_string(),
            "ui-bar-2".to_string(),
            "ui-bar-3".to_string(),
            "ui-bar-4".to_string(),
        ],
        18,
    ));

    animations.push((
        "ui-crosshair",
        vec![
            "ui-crosshair-1".to_string(),
            "ui-crosshair-2".to_string(),
            "ui-crosshair-3".to_string(),
            "ui-crosshair-4".to_string(),
        ],
        24,
    ));

    animations.push((
        "ui-crosshair-inactive",
        vec!["ui-crosshair-inactive".to_string()],
        1,
    ));

    image_assets.load_animations(&animations);
}
