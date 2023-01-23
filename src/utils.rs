/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use rand::Rng;

use bevy::{
    math::{ivec3, IVec3, Vec4},
    prelude::Color
};

use crate::{rule::Rule};

pub fn is_in_bounds(position: IVec3, bounds: i32) -> bool {
    position.x < bounds && position.y < bounds && position.z < bounds
}

// wrap around coordinates outside of bounds
pub fn wrap(position: IVec3, bounds: i32) -> IVec3 {
    (position + bounds) % bounds
}

pub fn distance_to_center(cell_position: IVec3, rule: &Rule) -> f32 {
    let cell_position = cell_position - rule.center();
    let max = rule.bounding_size as f32 / 2.0;

    cell_position.as_vec3().length() / max
}

pub fn spawn_noise<F: FnMut(IVec3)>(center: IVec3, radius: i32, amount: usize, mut f: F) {
    let mut random = rand::thread_rng();

    (0..amount).for_each(|_| {
        f(center + ivec3(
            random.gen_range(-radius..=radius),
            random.gen_range(-radius..=radius),
            random.gen_range(-radius..=radius)
        ))
    });
}

pub fn spawn_noise_default<F: FnMut(IVec3)>(center: IVec3, f: F) {
    spawn_noise(center, 6, 12 * 12 * 12, f)
}

pub fn lerp_colour(colour_1: Color, colour_2: Color, dt: f32) -> Color {
    let colour_1: Vec4 = colour_1.into();
    let colour_2: Vec4 = colour_2.into();
    let dt = dt.clamp(0.0, 1.0);

    ((1.0 - dt) * colour_1 + dt * colour_2).into()
}

pub fn index_to_position(index: usize, bounds: i32) -> IVec3 {
    ivec3(index as i32 % bounds, index as i32 / bounds % bounds, index as i32 / bounds / bounds)
}

pub fn position_to_index(position:IVec3, bounds: i32) -> usize {
    let x = position.x as usize;
    let y = position.y as usize;
    let z = position.z as usize;
    let bounds = bounds as usize;

    x + y * bounds + z * bounds * bounds
}
