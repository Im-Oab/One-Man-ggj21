use std::collections::HashMap;

use tetra::Context;

use crate::gameplay::particle_manager::{Particle, ParticleType};
use crate::image_assets::ImageAssets;
use crate::sprite::AnimationMultiTextures;

#[derive(Default)]
pub struct ExplosionParticleType {
    animations: HashMap<String, AnimationMultiTextures>,
}

impl ExplosionParticleType {
    #[must_use]
    pub fn new(_image_assets: &ImageAssets) -> Self {
        Self::default()
    }

    fn prepare_explosion_animation(&mut self, animation_name: &str, image_assets: &ImageAssets) {
        if !self.animations.contains_key(animation_name) {
            if let Some(default_animation) = image_assets.get_animation_object(animation_name) {
                self.animations
                    .insert(animation_name.to_owned(), default_animation);
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
                particle.sprite.play(animation);
            }
            None => {
                particle.active = true;
            }
        }

        if let Some(v) = particle.extra.get("flip_x") {
            let value = v.parse::<u8>().unwrap_or(0);

            if value == 1 {
                particle.sprite.flip_x(true);
            } else {
                particle.sprite.flip_x(false);
            }
        };
    }

    fn update(&self, particle: &mut Particle) {
        particle.sprite.update();

        if let Some(v) = particle.life_time.checked_sub(crate::ONE_FRAME.as_millis()) {
            particle.life_time = v
        } else {
            particle.life_time = 0;
            particle.active = false;
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
