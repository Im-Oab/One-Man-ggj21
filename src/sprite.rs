use tetra::graphics::animation::Animation;
use tetra::graphics::Color;
use tetra::graphics::{self, DrawParams, Drawable, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::Context;

use std::time::Duration;

use crate::image_assets::ImageAssets;

pub struct Sprite {
    /// True: Looping animation.
    /// False: stop animation at last frame.
    is_loop: bool,
    /// Currnt frame index for playing animation
    frame_index: usize,
    /// Current animation
    animation: AnimationMultiTextures,
    /// If frame duration exceed animation frame_length. It will change frame_index to the next frame.
    frame_duration: Duration,
    /// Internally use it for pausing animation when it reach last frame and is_loop = false.
    pause: bool,

    /// position
    pub position: Vec2<f32>,

    /// Use for scaling
    pub scale: Vec2<f32>,
    /// Use for setting origin of sprite.
    /// (0.0,0.0) = origin is top-left of sprite.
    /// (0.5,0.5) = origin is center of sprite.
    pub anchor: Vec2<f32>,
    /// Alpha value of the sprite. (0.0,disappear - 1.0, solid)
    alpha: f32,

    /// Last draw texture size
    pub size: Vec2<f32>,

    color: Color,
}

impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            is_loop: true,
            frame_index: 0,
            animation: AnimationMultiTextures::new(),
            frame_duration: Duration::from_millis(0),
            position: Vec2::zero(),
            scale: Vec2::one(),
            anchor: Vec2::new(0.5, 0.5),
            alpha: 1.0,
            pause: false,
            size: Vec2::one(),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// Set default values for all variables except animation.
    pub fn reset(&mut self) {
        self.is_loop = true;
        self.frame_index = 0;
        self.frame_duration = Duration::from_millis(0);
        self.position = Vec2::zero();
        self.scale = Vec2::one();
        self.anchor = Vec2::new(0.5, 0.5);
        self.alpha = 1.0;
        self.pause = false;
        self.animation = AnimationMultiTextures::new();
    }

    /// Get current animation name
    pub fn get_current_animation_name(&self) -> &String {
        &self.animation.name
    }

    /// Set animation loop flag
    pub fn set_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
    }

    /// Get animation loop flag
    pub fn is_loop(&self) -> bool {
        self.is_loop
    }

    /// If animation at last frame. This function will restart frame_index.
    pub fn continue_loop(&mut self) {
        let frames = self.animation.frames.len();
        self.frame_index = self.frame_index % frames;
    }

    pub fn show_texture(&mut self, texture_id: u128) {
        self.animation.clear();
        self.animation.add(texture_id);
        self.frame_index = 0;
    }

    /// Play new animation from first frame.
    pub fn play(&mut self, new_animation: &AnimationMultiTextures) {
        self.animation = new_animation.clone();
        self.frame_duration = Duration::from_millis(0);
        self.frame_index = 0;
        self.pause = false
    }

    /// Restart animation
    pub fn restart(&mut self) {
        self.frame_duration = Duration::from_millis(0);
        self.frame_index = 0;
        self.pause = false;
    }

    /// Use for update animation frame_index.
    fn advance(&mut self) -> bool {
        let frame_length = self.animation.frame_length;

        match self.frame_duration.checked_add(crate::ONE_FRAME) {
            Some(v) => self.frame_duration = v,
            None => self.frame_duration = Duration::from_millis(0),
        };

        if self.frame_duration >= frame_length && self.pause == false {
            while self.frame_duration >= frame_length {
                self.frame_duration -= frame_length;
                self.frame_index += 1;
            }

            if self.is_end_of_animation() == true {
                if self.is_loop == true {
                    self.continue_loop();
                } else {
                    self.frame_index = self.animation.frames.len() - 1;
                    self.pause = true;
                }

                return true;
            }
        }

        false
    }

    /// Draw sprite on screen.
    pub fn draw(
        &mut self,
        ctx: &mut Context,
        position: Vec2<f32>,
        rotation: f32,
        image_assets: &ImageAssets,
    ) {
        match self.animation.frames.get(self.frame_index) {
            Some(frame) => {
                match image_assets.get_by_id(&frame.texture_id) {
                    Some(texture) => {
                        let mut origin = self.get_origin(self.anchor.x, self.anchor.y);
                        let mut rect = frame.rect;
                        if rect.width == 0.0 && rect.height == 0.0 {
                            let width = texture.width() as f32;
                            let height = texture.height() as f32;
                            rect.width = width;
                            rect.height = height;

                            origin.x = width * self.anchor.x;
                            origin.y = height * self.anchor.y;
                        }

                        self.size.x = rect.width;
                        self.size.y = rect.height;

                        tetra::graphics::draw(
                            ctx,
                            texture,
                            DrawParams::new()
                                .position(position + self.position)
                                .origin(origin)
                                .rotation((rotation * 360.0).to_radians())
                                .scale(self.scale)
                                .clip(rect)
                                .color(Color::rgba(1.0, 1.0, 1.0, self.alpha)),
                        );
                    }
                    None => (),
                };
            }
            None => (),
        };
    }

    pub fn draw_with_clipping(
        &mut self,
        ctx: &mut Context,
        position: Vec2<f32>,
        rotation: f32,
        image_assets: &ImageAssets,
        clipping: Vec2<f32>,
    ) {
        match self.animation.frames.get(self.frame_index) {
            Some(frame) => {
                match image_assets.get_by_id(&frame.texture_id) {
                    Some(texture) => {
                        let mut origin = self.get_origin(self.anchor.x, self.anchor.y);
                        let mut rect = frame.rect;
                        if rect.width == 0.0 && rect.height == 0.0 {
                            let width = texture.width() as f32;
                            let height = texture.height() as f32;
                            rect.width = width;
                            rect.height = height;

                            origin.x = width * self.anchor.x;
                            origin.y = height * self.anchor.y;
                        }

                        self.size.x = rect.width;
                        self.size.y = rect.height;
                        rect.width = rect.width * clipping.x;
                        rect.height = rect.height * clipping.y;

                        tetra::graphics::draw(
                            ctx,
                            texture,
                            DrawParams::new()
                                .position(position)
                                .origin(origin)
                                .rotation((rotation * 360.0).to_radians())
                                .scale(self.scale)
                                .clip(rect)
                                .color(Color::rgba(1.0, 1.0, 1.0, self.alpha)),
                        );
                    }
                    None => (),
                };
            }
            None => (),
        };
    }

    pub fn update(&mut self) -> bool {
        self.advance()
    }

    pub fn is_end_of_animation(&self) -> bool {
        if self.pause && self.frame_index == self.animation.frames.len() - 1 {
            return true;
        }

        if self.animation.frames.len() == 0 {
        } else if self.frame_index > self.animation.frames.len() - 1 {
            return true;
        }

        false
    }

    pub fn get_total_frames(&self) -> usize {
        self.animation.frames.len()
    }

    pub fn get_current_frame_index(&self) -> usize {
        self.frame_index
    }
}

// scale
impl Sprite {
    pub fn flip_x(&mut self, b: bool) {
        if b {
            self.scale.x = self.scale.x.abs() * -1.0;
        } else {
            self.scale.x = self.scale.x.abs();
        }
    }

    pub fn flip_y(&mut self, b: bool) {
        if b {
            self.scale.y = self.scale.y.abs() * -1.0;
        } else {
            self.scale.y = self.scale.y.abs();
        }
    }
}

// Anchor
impl Sprite {
    pub fn set_anchor(&mut self, x: f32, y: f32) {
        self.anchor.x = x.min(1.0).max(0.0);
        self.anchor.y = y.min(1.0).max(0.0);
    }

    fn get_origin(&self, x: f32, y: f32) -> Vec2<f32> {
        if self.animation.frames.len() == 0 {
            return Vec2::zero();
        }

        let bound = self.animation.frames[self.frame_index].rect;

        let x = bound.width as f32 * x;
        let y = bound.height as f32 * y;

        Vec2::new(x, y)
    }
}

// Color
impl Sprite {
    pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
        self.color = Color::rgba(r, g, b, self.alpha);
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
        self.color.a = alpha;
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }
}

/// Animation object that consist of multiple FrameRectangle objects.
pub struct AnimationMultiTextures {
    /// Length between frames.
    /// Playing animation will use this value for change between frames.
    pub frame_length: Duration,
    /// List of all frames in this animation.
    pub frames: Vec<FrameRectangle>,

    pub name: String,
}

impl Clone for AnimationMultiTextures {
    fn clone(&self) -> AnimationMultiTextures {
        AnimationMultiTextures {
            frame_length: self.frame_length.clone(),
            frames: self.frames.clone(),
            name: self.name.clone(),
        }
    }
}

impl AnimationMultiTextures {
    pub fn new() -> AnimationMultiTextures {
        AnimationMultiTextures {
            frame_length: Duration::from_millis(1000 / 8),
            frames: Vec::new(),
            name: String::from(""),
        }
    }

    pub fn new_with_frames(frames: Vec<u128>) -> AnimationMultiTextures {
        let mut anim_obj = AnimationMultiTextures {
            frame_length: Duration::from_millis(1000 / 12),
            frames: Vec::new(),
            name: String::from(""),
        };

        anim_obj.add_frames(&frames);

        anim_obj
    }
    pub fn clear(&mut self) {
        self.frames.clear();
    }
    /// Create a frame with no-size and add it to animation frames.
    /// Frame with no-size will show as full texture size.
    pub fn add(&mut self, texture_id: u128) {
        self.add_with_rectangle(texture_id, Rectangle::new(0.0, 0.0, 0.0, 0.0));
    }

    /// Create and Append multiple frames with no-size into animation frames
    pub fn add_frames(&mut self, frames: &Vec<u128>) {
        for texture_id in frames.iter() {
            self.add(*texture_id);
        }
    }

    /// Add one frame with size into animation frames.
    pub fn add_with_rectangle(&mut self, texture_id: u128, rect: Rectangle) {
        self.frames.push(FrameRectangle::new(texture_id, rect));
    }
}

/// Frame area inside texture.
/// It uses for showing animation on CustomSprite
#[derive(Copy)]
pub struct FrameRectangle {
    /// Reference to texture using texture_id. Require ImageAssets for getting actual Texture.
    pub texture_id: u128,
    /// Frame area
    pub rect: Rectangle,
}

impl FrameRectangle {
    pub fn new(texture_id: u128, rect: Rectangle) -> FrameRectangle {
        FrameRectangle {
            texture_id: texture_id,
            rect: rect,
        }
    }
}

impl Clone for FrameRectangle {
    fn clone(&self) -> FrameRectangle {
        FrameRectangle {
            texture_id: self.texture_id,
            rect: self.rect,
        }
    }
}
