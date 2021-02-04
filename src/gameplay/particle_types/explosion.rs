use std::collections::HashMap;

use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::particle_manager::{Particle, ParticleSpawnNode, ParticleType};
use crate::image_assets::ImageAssets;
use crate::sprite::{AnimationMultiTextures, Sprite};

pub struct ExplosionParticleType {
    animations: HashMap<String, AnimationMultiTextures>,
}

impl ExplosionParticleType {
    pub fn new(_image_assets: &ImageAssets) -> ExplosionParticleType {
        ExplosionParticleType {
            animations: HashMap::new(),
        }
    }

    fn prepare_explosion_animation(&mut self, animation_name: &str, image_assets: &ImageAssets) {
        if self.animations.contains_key(animation_name) == false {
            let key = String::from(animation_name);
            match image_assets.get_animation_object(&key) {
                Some(default_animation) => {
                    self.animations.insert(key, default_animation);
                }
                None => (),
            }
        }
    }
}

impl ParticleType for ExplosionParticleType {
    fn particle_type_id(&self) -> u128 {
        1
    }

    fn init(&mut self, particle: &mut Particle, image_assets: &ImageAssets) {
        let animation_name = match particle.extra.get("idle_animation") {
            Some(name) => name.as_str(),
            None => "",
        };

        self.prepare_explosion_animation(animation_name, image_assets);
        match self.animations.get(animation_name) {
            Some(animation) => {
                particle.life_time = 1500;
                particle.sprite.play(&animation);
            }
            None => {
                particle.active = true;
            }
        }

        match particle.extra.get("flip_x") {
            Some(v) => {
                let value = v.parse::<u8>().unwrap_or(0);

                if value == 1 {
                    particle.sprite.flip_x(true);
                } else {
                    particle.sprite.flip_x(false);
                }
            }
            None => (),
        };
    }

    fn update(&self, particle: &mut Particle) {
        particle.sprite.update();

        match particle.life_time.checked_sub(crate::ONE_FRAME.as_millis()) {
            Some(v) => particle.life_time = v,
            None => {
                particle.life_time = 0;
                particle.active = false;
            }
        };

        if particle.sprite.is_end_of_animation() {
            particle.active = false;
        }
    }

    fn draw(&self, ctx: &mut Context, particle: &mut Particle, image_assets: &ImageAssets) {
        particle
            .sprite
            .draw(ctx, particle.position, particle.rotation, image_assets);
    }
}
