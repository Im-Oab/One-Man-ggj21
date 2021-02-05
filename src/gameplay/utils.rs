use tetra::math::Vec2;

use rand::prelude::*;

pub fn clamp_position_inside_camera_area(position: &mut Vec2<f32>) {
    let camera_position = crate::CAMERA_POSITION.lock().unwrap();

    let half_size = 16.0 / 2.0;
    let left = camera_position.x - crate::SCREEN_WIDTH / 2.0 - half_size;
    let right = camera_position.x + crate::SCREEN_WIDTH / 2.0 + half_size;
    let up = camera_position.y - crate::SCREEN_HEIGHT / 2.0;
    let down = camera_position.y + crate::SCREEN_HEIGHT / 2.0;

    position.x = position.x.max(left).min(right);
    position.y = position.y.max(up).min(down);
}

#[must_use]
pub fn is_inside_camera_area(position: Vec2<f32>, radius: f32) -> bool {
    let camera_position = crate::CAMERA_POSITION.lock().unwrap();

    let left = camera_position.x - crate::SCREEN_WIDTH / 2.0 - radius;
    let right = camera_position.x + crate::SCREEN_WIDTH / 2.0 + radius;

    if position.x >= left && position.x <= right {
        let up = camera_position.y - crate::SCREEN_HEIGHT / 2.0 - radius;
        let down = camera_position.y + crate::SCREEN_HEIGHT / 2.0 + radius;

        if position.y >= up && position.y <= down {
            return true;
        }
    }
    false
}

/// Random position inside camera.
/// top,left,width,height values in percentage. 0.0 - 1.0
#[must_use]
pub fn random_position_inside_camera_area(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
) -> Vec2<f32> {
    let camera_position = crate::CAMERA_POSITION.lock().unwrap();

    let width = width.min(crate::SCREEN_WIDTH);
    let height = height.min(crate::SCREEN_HEIGHT);

    let top = top.mul_add(
        crate::SCREEN_HEIGHT,
        camera_position.y - crate::SCREEN_HEIGHT / 2.0,
    );
    let left = left.mul_add(
        crate::SCREEN_WIDTH,
        camera_position.x - crate::SCREEN_WIDTH / 2.0,
    );

    let value_x = random::<f32>() * (crate::SCREEN_WIDTH * width);
    let value_y = random::<f32>() * (crate::SCREEN_HEIGHT * height);

    let mut random_position = Vec2::new(left + value_x, top + value_y);

    {
        let left_max = camera_position.x - crate::SCREEN_WIDTH / 2.0;
        let right_min = camera_position.x + crate::SCREEN_WIDTH / 2.0;
        let up_max = camera_position.y - crate::SCREEN_HEIGHT / 2.0;
        let down_min = camera_position.y + crate::SCREEN_HEIGHT / 2.0;

        random_position.x = random_position.x.max(left_max).min(right_min);
        random_position.y = random_position.y.max(up_max).min(down_min);
    }

    random_position
}

#[must_use]
pub fn convert_screen_position_to_world_position(screen_position: Vec2<f32>) -> Vec2<f32> {
    let camera_position = crate::CAMERA_POSITION.lock().unwrap();
    screen_position + *camera_position
}

#[must_use]
pub const fn distance_sqr(x1: i128, y1: i128, x2: i128, y2: i128) -> i128 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let distance = (dx * dx) + (dy * dy);

    if distance <= 0 {
        return i128::MAX;
    }

    distance
}

#[must_use]
pub fn lerp(a: f32, b: f32, v: f32) -> f32 {
    let clamp_v = v.max(0.0).min(1.0);

    a.mul_add(1_f32 - clamp_v, b * clamp_v)
}

#[must_use]
pub fn angle_lerp(angle_1: f32, angle_2: f32, t: f32) -> f32 {
    let mut angle_1 = angle_1;
    let mut angle_2 = angle_2;

    angle_1 %= 1.0;
    angle_2 %= 1.0;

    if (angle_1 - angle_2).abs() > 0.5 {
        if angle_1 > angle_2 {
            angle_2 += 1.0;
        } else {
            angle_1 += 1.0;
        }
    }

    lerp(angle_1, angle_2, t) % 1.0
}
