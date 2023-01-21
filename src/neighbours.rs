/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use bevy::math::{const_ivec3, IVec3};

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
