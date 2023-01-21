/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use std::ops::RangeInclusive;

use bevy::prelude::Color;

use crate::{
    neighbours::NeighbourMethod,
    utils
};

#[allow(dead_code)]
#[derive(Clone)]
pub enum Value {
    Single(u8),
    Range(RangeInclusive<u8>),
    Singles(Vec<u8>)
}

impl Value {
    pub fn in_range(&self, value: u8) -> bool {
        match self {
            Value::Single(single) => value == *single,
            Value::Range(range) => value < *range.end() && value > *range.start(),
            Value::Singles(singles) => singles.iter().any(|v| *v == value)
        }
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
    pub neighbour_method: NeighbourMethod,
    pub bounding_size: i32,
    pub colour_method: ColourMethod
}

impl Rule {
    pub(crate) fn get_bounding_ranges(&self) -> (RangeInclusive<i32>, RangeInclusive<i32>, RangeInclusive<i32>) {
        let x_range = -self.bounding_size..=self.bounding_size;
        let y_range = -self.bounding_size..=self.bounding_size;
        let z_range = -self.bounding_size..=self.bounding_size;

        (x_range, y_range, z_range)
    }
}
