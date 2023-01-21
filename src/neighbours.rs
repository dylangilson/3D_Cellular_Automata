/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use bevy::math::{const_ivec3, IVec3};

#[allow(dead_code)]
#[derive(Clone)]
pub enum NeighbourMethod {
    Moore,
    VonNeuman
}

impl NeighbourMethod {
    pub fn get_neighbour_iter(&self) -> &'static [IVec3] {
        match self {
            NeighbourMethod::Moore => &MOORE_NEIGHBOURS[..],
            NeighbourMethod::VonNeuman => &VONNEUMAN_NEIGHBOURS[..]
        }
    }
}

pub static MOORE_NEIGHBOURS: [IVec3; 26] = [
    const_ivec3!([-1, -1, -1]),
    const_ivec3!([0, -1, -1]),
    const_ivec3!([1, -1, -1]),
    const_ivec3!([-1, 0, -1]),
    const_ivec3!([0, 0, -1]),
    const_ivec3!([1, 0, -1]),
    const_ivec3!([-1, 1, -1]),
    const_ivec3!([0, 1, -1]),
    const_ivec3!([1, 1, -1]),
    const_ivec3!([-1, -1, 0]),
    const_ivec3!([0, -1, 0]),
    const_ivec3!([1, -1, 0]),
    const_ivec3!([-1, 0, 0]),
    const_ivec3!([1, 0, 0]),
    const_ivec3!([-1, 1, 0]),
    const_ivec3!([0, 1, 0]),
    const_ivec3!([1, 1, 0]),
    const_ivec3!([-1, -1, 1]),
    const_ivec3!([0, -1, 1]),
    const_ivec3!([1, -1, 1]),
    const_ivec3!([-1, 0, 1]),
    const_ivec3!([0, 0, 1]),
    const_ivec3!([1, 0, 1]),
    const_ivec3!([-1, 1, 1]),
    const_ivec3!([0, 1, 1]),
    const_ivec3!([1, 1, 1])
];

pub static VONNEUMAN_NEIGHBOURS: [IVec3; 6] = [
    const_ivec3!([1, 0, 0]),
    const_ivec3!([-1, 0, 0]),
    const_ivec3!([0, 1, 0]),
    const_ivec3!([0, -1, 0]),
    const_ivec3!([0, 0, -1]),
    const_ivec3!([0, 0, 1])
];
