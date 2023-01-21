/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use std::collections::HashMap;
use rand::Rng;

use bevy::math::{ivec3, IVec3};

use crate::{rule::Rule, State};

// wrap around coordinates outside of bounds
pub fn keep_in_bounds(bounds: i32, position: &mut IVec3) {
    if position.x < -bounds + 1 {
        position.x = bounds - 1;
    } else if position.x > bounds - 1 {
        position.x = -bounds + 1;
    }

    if position.y < -bounds + 1 {
        position.y = bounds - 1;
    } else if position.y > bounds - 1 {
        position.y = -bounds + 1;
    }

    if position.z < -bounds + 1 {
        position.z = bounds - 1;
    } else if position.z > bounds - 1 {
        position.z = -bounds + 1;
    }
}

pub fn spawn_noise(states: &mut HashMap<IVec3, State>, rule: &Rule) {
    let mut random = rand::thread_rng();
    let spawn_size = 6;

    (0..199).for_each(|_i| {
        states.insert(ivec3(random.gen_range(-spawn_size..=spawn_size), random.gen_range(-spawn_size..=spawn_size),
            random.gen_range(-spawn_size..=spawn_size)), State::new(rule.start_state_value));
    });
}
