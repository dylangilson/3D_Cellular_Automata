/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use std::collections::HashMap;
use rand::Rng;

use bevy::{
    math::{ivec3, IVec3},
    prelude::Color
};

use crate::{
    rule::Rule,
    CellState
};

// wrap around coordinates outside of bounds
pub fn keep_in_bounds(bounds: i32, position: &mut IVec3) {
    if position.x <= -bounds {
        position.x = bounds - 1;
    } else if position.x >= bounds {
        position.x = -bounds + 1;
    }

    if position.y <= -bounds {
        position.y = bounds - 1;
    } else if position.y >= bounds {
        position.y = -bounds + 1;
    }

    if position.z <= -bounds {
        position.z = bounds - 1;
    } else if position.z >= bounds {
        position.z = -bounds + 1;
    }
}

pub fn distance_to_center(cell_position: IVec3, rule: &Rule) -> f32 {
    let max = rule.bounding_size as f32;

    cell_position.as_vec3().length() / max
}

pub fn spawn_noise(states: &mut HashMap<IVec3, CellState>, rule: &Rule) {
    let mut random = rand::thread_rng();
    let spawn_size = 6;

    (0..12 * 12 * 12).for_each(|_i| {
        let position = ivec3(
            random.gen_range(-spawn_size..=spawn_size),
            random.gen_range(-spawn_size..=spawn_size),
            random.gen_range(-spawn_size..=spawn_size)
        );
        let distance = distance_to_center(position, rule);

        states.insert(position, CellState::new(rule.states, 0, distance));
    });
}

pub fn spawn_noise_small(states: &mut HashMap<IVec3, CellState>, rule: &Rule) {
    let mut random = rand::thread_rng();
    let spawn_size = 1;

    (0..12 * 12 * 12).for_each(|_i| {
        let position = ivec3(
            random.gen_range(-spawn_size..=spawn_size),
            random.gen_range(-spawn_size..=spawn_size),
            random.gen_range(-spawn_size..=spawn_size)
        );
        let distance = distance_to_center(position, rule);

        states.insert(position, CellState::new(rule.states, 0, distance));
    });
}

pub fn lerp_colour(colour_1: Color, colour_2: Color, dt: f32) -> Color {
    let colour_1 = colour_1.as_rgba_f32();
    let colour_2 = colour_2.as_rgba_f32();
    let dt = dt.max(0.0).min(1.0);
    let inversion = 1.0 - dt;
    let lerped = [
        colour_1[0] * dt + colour_2[0] * inversion,
        colour_1[1] * dt + colour_2[1] * inversion,
        colour_1[2] * dt + colour_2[2] * inversion,
        colour_1[3] * dt + colour_2[3] * inversion
    ];

    Color::rgba(lerped[0], lerped[1], lerped[2], lerped[3])
}
