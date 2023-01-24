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

// wrap around coordinates outside of bounds
pub fn wrap(position: IVec3, bounds: i32) -> IVec3 {
    (position + bounds) % bounds
}

// distance from cell to origin
pub fn distance_to_center(cell_position: IVec3, bounds: i32) -> f32 {
    let cell_position = cell_position - center(bounds);
    let max = bounds as f32 / 2.0;

    cell_position.as_vec3().length() / max
}

// spawn cubes in within radius from origin
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

// spawn noise wrapper with default values
pub fn spawn_noise_default<F: FnMut(IVec3)>(center: IVec3, f: F) {
    spawn_noise(center, 6, 12 * 12 * 12, f)
}

// linear interpolation between colour 1 and colour 2
pub fn lerp_colour(colour_1: Color, colour_2: Color, dt: f32) -> Color {
    let colour_1: Vec4 = colour_1.into();
    let colour_2: Vec4 = colour_2.into();
    let dt = dt.clamp(0.0, 1.0);

    ((1.0 - dt) * colour_1 + dt * colour_2).into()
}

// convert index to xyz position
pub fn index_to_position(index: usize, bounds: i32) -> IVec3 {
    ivec3(index as i32 % bounds, index as i32 / bounds % bounds, index as i32 / bounds / bounds)
}

// convert xyz position to index
pub fn position_to_index(position:IVec3, bounds: i32) -> usize {
    let x = position.x as usize;
    let y = position.y as usize;
    let z = position.z as usize;
    let bounds = bounds as usize;

    x + y * bounds + z * bounds * bounds
}

// get xyz position of center of bounds
pub fn center(bounds: i32) -> IVec3 {
    let center = bounds / 2;

    ivec3(center, center, center)
}
