/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use std::ops::RangeInclusive;

use bevy::{
    math::{ivec3, IVec3},
    prelude::Color
};

use crate::{
    neighbours::NeighbourMethod,
    utils
};


#[derive(Clone, Copy)]
pub struct Value ([bool; 27]);

#[allow(dead_code)]
impl Value {
    pub fn new(indices: &[u8]) -> Self {
        let mut result = Value([false; 27]);

        for index in indices {
            result.0[*index as usize] = true;
        }

        result
    }

    pub fn from_range(indices: RangeInclusive<u8>) -> Self {
        let mut result = Value([false; 27]);

        for index in indices {
            result.0[index as usize] = true;
        }

        result
    }

    pub fn in_range(&self, value: u8) -> bool {
        self.0[value as usize]
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ColourMethod {
    Single(Color),
    StateLerp(Color, Color),
    DistanceToCenter(Color, Color),
    Neighbour(Color, Color)
}

impl ColourMethod {
    pub fn colour(&self, states: u8, state: u8, neighbours: u8, distance_to_center: f32) -> Color {
        match self {
            ColourMethod::Single(c) => *c,
            ColourMethod::StateLerp(c1, c2) => {
                let dt = state as f32 / states as f32;

                utils::lerp_colour(*c1, *c2, dt)
            },
            ColourMethod::DistanceToCenter(center_c, bounds_c) => {
                utils::lerp_colour(*center_c, *bounds_c, distance_to_center)
            },
            ColourMethod::Neighbour(c1, c2) => {
                let dt = neighbours as f32 / 26f32;

                utils::lerp_colour(*c1, *c2, dt)
            }
        }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub survival_rule: Value,
    pub birth_rule: Value,
    pub states: u8,
    pub bounding_size: i32,
    pub colour_method: ColourMethod,
    pub neighbour_method: NeighbourMethod
}

impl Rule {
    pub fn get_bounding_ranges(&self) -> (RangeInclusive<i32>, RangeInclusive<i32>, RangeInclusive<i32>) {
        let x_range = 0..=self.bounding_size - 1;
        let y_range = 0..=self.bounding_size - 1;
        let z_range = 0..=self.bounding_size - 1;

        (x_range, y_range, z_range)
    }

    pub fn center(&self) -> IVec3 {
        let center = self.bounding_size / 2;

        ivec3(center, center, center)
    }
}
