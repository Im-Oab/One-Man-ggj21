use tetra::graphics::{self, Camera, Color, GeometryBuilder, Mesh, Rectangle, DrawParams};
use tetra::graphics::text::{Font, Text};
use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::player::WeaponType;
use crate::image_assets::ImageAssets;
use crate::sprite::Sprite;

pub struct UI {
    circle: Sprite,
    melee: Sprite,
    range: Sprite,
    crosshair: Sprite,
    inactive_crosshair: Sprite,
    energy: Sprite,
    z_button: Sprite,

    game_over_text: Option<Text>,
    restart_text: Option<Text>,

    level_cleared_text: Option<Text>,
    credits_text: Option<Text>,

    intro_text: Option<Text>,
    warning: Option<Text>,
}

impl UI {
    pub fn new() -> UI {
        UI {
            circle: Sprite::new(),
            melee: Sprite::new(),
            range: Sprite::new(),
            crosshair: Sprite::new(),
            inactive_crosshair: Sprite::new(),
            energy: Sprite::new(),
            z_button: Sprite::new(),
            game_over_text : None,
            restart_text: None,
            level_cleared_text: None,
            credits_text: None,

            intro_text: None,
            warning: None,
        }
    }

    pub fn setup(&mut self, ctx: &mut Context, image_assets: &ImageAssets) {
        match image_assets.get_animation_object("ui-circle") {
            Some(animation) => {
                self.circle.play(&animation);
                self.circle.scale = Vec2::new(1.4, 1.4);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-z") {
            Some(animation) => {
                self.z_button.play(&animation);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-weapon-melee") {
            Some(animation) => {
                self.melee.play(&animation);
                self.melee.scale = Vec2::new(1.2, 1.2);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-weapon-range") {
            Some(animation) => {
                self.range.play(&animation);
                self.range.scale = Vec2::new(1.2, 1.2);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-crosshair") {
            Some(animation) => {
                self.crosshair.play(&animation);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-crosshair-inactive") {
            Some(animation) => {
                self.inactive_crosshair.play(&animation);
            }
            None => (),
        };

        match image_assets.get_animation_object("ui-bar") {
            Some(animation) => {
                self.energy.play(&animation);
            }
            None => (),
        };

        let font = match Font::vector(ctx, "./resources/fonts/D-DINCondensed.ttf", 36.0) {
            Ok(v) => v,
            Err(_) => panic!("Load font error for loading screen"),
        };
        self.intro_text = Some(Text::new("A CREATURE BREAKOUT FROM CONTAINMENT. YOU MUST FIND IT.", font.clone()));
        self.warning = Some(Text::new("IT'S NEARBY", font.clone()));

        let font = match Font::vector(ctx, "./resources/fonts/D-DINCondensed.ttf", 128.0) {
            Ok(v) => v,
            Err(_) => panic!("Load font error for loading screen"),
        };

        
        self.game_over_text = Some(Text::new("GAMEOVER", font.clone()));
        self.level_cleared_text = Some(Text::new("END", font));

        let font = match Font::vector(ctx, "./resources/fonts/D-DINCondensed.ttf", 32.0) {
            Ok(v) => v,
            Err(_) => panic!("Load font error for loading screen"),
        };
        self.restart_text = Some(Text::new("PRESS 'Z' TO RESTART", font.clone()));
        self.credits_text = Some(Text::new("A GAME BY OAB", font));
    }

    pub fn update(&mut self, _ctx: &mut Context) {
        self.circle.update();
        self.z_button.update();
        self.crosshair.update();
        self.energy.update();
    }

    pub fn draw_intro(&mut self, ctx: &mut Context, position: Vec2<f32>)
    {
        UI::draw_3_colors_text(ctx, position, self.intro_text.as_ref().unwrap());
    }

    pub fn draw_warning(&mut self, ctx: &mut Context, position: Vec2<f32>)
    {
        UI::draw_3_colors_text(ctx, position, self.warning.as_ref().unwrap());
    }

    pub fn draw_game_over(&mut self,
        ctx: &mut Context        
        )
    {

        let position = Vec2::new(
            ((crate::SCREEN_WIDTH - self.game_over_text.as_ref().unwrap().get_bounds(ctx).unwrap().width)/2.0).ceil(),
            (crate::SCREEN_HEIGHT * 0.2).ceil()
        );   

        UI::draw_3_colors_text(ctx, position, self.game_over_text.as_ref().unwrap());

        let position = Vec2::new(
            ((crate::SCREEN_WIDTH - self.restart_text.as_ref().unwrap().get_bounds(ctx).unwrap().width)/2.0).ceil(),
            (crate::SCREEN_HEIGHT * 0.7).ceil()
        );   
        UI::draw_3_colors_text(ctx, position, self.restart_text.as_ref().unwrap());
    }

    pub fn draw_level_cleared(&mut self,
        ctx: &mut Context
        
        )
    {

        let position = Vec2::new(
            ((crate::SCREEN_WIDTH - self.level_cleared_text.as_ref().unwrap().get_bounds(ctx).unwrap().width)/2.0).ceil(),
            (crate::SCREEN_HEIGHT * 0.2).ceil()
        );   

        UI::draw_3_colors_text(ctx, position, self.level_cleared_text.as_ref().unwrap());

        let position = Vec2::new(
            ((crate::SCREEN_WIDTH - self.credits_text.as_ref().unwrap().get_bounds(ctx).unwrap().width)/2.0).ceil(),
            (crate::SCREEN_HEIGHT * 0.7).ceil()
        );   
        UI::draw_3_colors_text(ctx, position, self.credits_text.as_ref().unwrap());
    }

    pub fn draw_weapon(
        &mut self,
        ctx: &mut Context,
        image_assets: &ImageAssets,
        weapon_type: &WeaponType,
    ) {
        self.circle.draw(
            ctx,
            Vec2::new(24.0, crate::SCREEN_HEIGHT - 24.0),
            0.0,
            image_assets,
        );

        self.z_button.draw(
            ctx,
            Vec2::new(36.0, crate::SCREEN_HEIGHT - 10.0),
            0.0,
            image_assets,
        );

        match weapon_type {
            WeaponType::Melee => {
                self.melee.draw(
                    ctx,
                    Vec2::new(24.0, crate::SCREEN_HEIGHT - 24.0),
                    0.0,
                    image_assets,
                );
            }
            WeaponType::Range => {
                self.range.draw(
                    ctx,
                    Vec2::new(24.0, crate::SCREEN_HEIGHT - 24.0),
                    0.0,
                    image_assets,
                );
            }
        }
    }

    pub fn draw_energy_bar(
        &mut self,
        ctx: &mut Context,
        image_assets: &ImageAssets,
        energy_percentage: f32,
    ) {
        let position = Vec2::new(40.0, crate::SCREEN_HEIGHT - 30.0);

        match image_assets.get("ui-bar-bg") {
            Some(texture) => {
                graphics::draw(ctx, texture, position);
            }
            None => (),
        };

        self.energy.draw_with_clipping(
            ctx,
            Vec2::new(72.0, crate::SCREEN_HEIGHT - 16.0),
            0.0,
            image_assets,
            Vec2::new(energy_percentage, 1.0),
        );

        match image_assets.get("ui-bar-fg") {
            Some(texture) => {
                graphics::draw(ctx, texture, position);
            }
            None => (),
        };
    }

    pub fn draw_crosshair(
        &mut self,
        ctx: &mut Context,
        image_assets: &ImageAssets,
        weapon_type: &WeaponType,
        crosshair_position: &Vec2<f32>,
    ) {
        match weapon_type {
            WeaponType::Melee => {
                self.inactive_crosshair
                    .draw(ctx, *crosshair_position, 0.0, image_assets);
            }
            WeaponType::Range => {
                self.crosshair
                    .draw(ctx, *crosshair_position, 0.0, image_assets);
            }
        }
    }

    fn draw_3_colors_text(ctx: &mut Context, position: Vec2<f32>, draw_text: &Text) {
        {
            let params = DrawParams::new()
                .position(Vec2::new(position.x - 2.0, position.y - 2.0))
                .color(Color::WHITE);
    
            graphics::draw(ctx, draw_text, params);
        }
    
        {
            let params = DrawParams::new()
                .position(Vec2::new(position.x + 2.0, position.y + 2.0))
                .color(Color::rgba8(255, 20, 20, 255));
    
            graphics::draw(ctx, draw_text, params);
        }
    
        {
            let params = DrawParams::new()
                .position(Vec2::new(position.x, position.y))
                .color(Color::BLACK);
    
            graphics::draw(ctx, draw_text, params);
        }
    }
}
