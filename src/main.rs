/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 19, 2023
 */

use std::{
    ops::RangeInclusive,
    sync::{Arc, Mutex}
};

use std::collections::HashMap;
use rand::Rng;

use bevy::{
    // diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::{const_ivec3, ivec3, vec3},
    prelude::*,
    render::view::NoFrustumCulling
};

mod cell_renderer;
use cell_renderer::*;

mod neighbours;
use neighbours::MOORE_NEIGHBOURS;

mod rule;
use rule::*;

mod utils;
use utils::keep_in_bounds;

#[derive(Debug)]
pub struct State {
    value: u8
}

impl State {
    pub fn new(value: u8) -> Self {
        State {
            value
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn().insert_bundle((meshes.add(Mesh::from(shape::Cube {size: 0.9})), Transform::from_xyz(0.0, 0.0, 0.0),
                               GlobalTransform::default(),
                               InstanceMaterialData((1..=10).flat_map(|x| (1..=100).map(move |y| (x as f32 / 10.0, y as f32 / 10.0)))
            .map(|(x, y)| InstanceData {
                position: Vec3::new(x * 10.0 - 5.0, y * 10.0 - 5.0, 0.0),
                scale: 1.0,
                colour: Color::hsla(x * 360., y, 0.5, 1.0).as_rgba_f32(),
            }).collect()
        ), Visibility::default(), ComputedVisibility::default(), NoFrustumCulling)
    );

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube {size: 1.0})),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn main() {
    let rule = Rule {
        // survival_rule: Value::Singles(vec![3, 6, 9]),
        // birth_rule: Value::Singles(vec![4, 8, 9]),

        // survival_rule: Value::Range(9..=26),
        // birth_rule: Value::Range(5..=7),

        // survival_rule: Value::Single(4),
        // birth_rule: Value::Single(4),

        survival_rule: Value::Range(8..=26),
        birth_rule: Value::Singles(vec![4, 12, 13, 15]),
        states: 5,
        start_state_value: 5,
        bounding: 50
    };

    App::new().add_plugins(DefaultPlugins).add_plugin(CellMaterialPlugin).insert_resource(rule)
        .add_startup_system(setup).run();
}
