use tetra::graphics::{self, Color, GeometryBuilder, Mesh, Rectangle, ShapeStyle};
use tetra::math::Vec2;
use tetra::Context;

use crate::gameplay::bullet_pool::{Bullet, BulletOwner, BulletType};
use crate::image_assets::ImageAssets;
pub struct ConstantVelocityBulletType {
    // skin: Mesh,
}

impl ConstantVelocityBulletType {
    pub fn new(ctx: &mut Context) -> ConstantVelocityBulletType {
        let skin = GeometryBuilder::new()
            .set_color(Color::BLUE)
            .circle(ShapeStyle::Fill, Vec2::zero(), 6.0)
            .unwrap()
            .build_mesh(ctx)
            .unwrap();
        ConstantVelocityBulletType{
            // skin: skin
        }
    }
}

impl BulletType for ConstantVelocityBulletType {
    /// Return type id of the BulletType.
    /// This value have to be unique between BulletType
    fn bullet_type_id(&self) -> i32 {
        1
    }

    /// Setup bullet data.
    /// It will use first split('|') of bullet.extra for animation_name.
    /// If no animation. It will set bullet.active to false
    fn setup(&mut self, bullet: &mut Bullet, image_assets: &ImageAssets) {
        bullet.life_time = 100;
        bullet.health = 2;

        let idle_animation_name = match bullet.extra.get("idle_animation") {
            Some(name) => name.as_str(),
            None => "",
        };

        match image_assets.get_animation_object(idle_animation_name) {
            Some(animation) => {
                bullet.sprite.play(&animation);
            }
            None => (),
        };
    }

    /// Use for updating bullet position.
    fn update(&self, bullet: &mut Bullet) {
        bullet.position.x += (bullet.rotation * 360.0).to_radians().cos() * bullet.speed;
        bullet.position.y += (bullet.rotation * 360.0).to_radians().sin() * bullet.speed;
        bullet.sprite.update();
    }

    /// Draw bullet on screen. It will be called from inside Bullet.update()
    fn draw(&self, ctx: &mut Context, image_assets: &mut ImageAssets, bullet: &mut Bullet) {
        bullet
            .sprite
            .draw(ctx, bullet.position, bullet.rotation, image_assets);
    }
}
