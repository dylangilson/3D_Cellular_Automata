/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 19, 2023
 */

use bevy::{
    prelude::*,
    render::view::NoFrustumCulling
};

mod cell_renderer;
use cell_renderer::*;

mod cells_multithreaded;
use cells_multithreaded::*;

mod neighbours;
use neighbours::NeighbourMethod;

mod rotating_camera;
use rotating_camera::{RotatingCamera, RotatingCameraPlugin};

mod rule;
use rule::*;

mod utils;

#[derive(Debug)]
pub struct CellState {
    value: u8,
    neighbours: u8,
    distance_to_center: f32
}

impl CellState {
    pub fn new(value: u8, neighbours: u8, distance_to_center: f32) -> Self {
        CellState {
            value,
            neighbours,
            distance_to_center
        }
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube {size: 1.0})),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        InstanceMaterialData((1..=10)
            .flat_map(|x| (1..=100).map(move |y| (x as f32 / 10.0, y as f32 / 10.0)))
            .map(|(x, y)| InstanceData {
                position: Vec3::new(x * 10.0 - 5.0, y * 10.0 - 5.0, 0.0),
                scale: 1.0,
                colour: Color::hsla(x * 360., y, 0.5, 1.0).as_rgba_f32(),
            })
            .collect()
        ),
        Visibility::default(),
        ComputedVisibility::default(),
        NoFrustumCulling)
    );

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    }).insert(RotatingCamera::default());
}

fn main() {
    let rule = Rule {
        bounding_size: 25,

        // builder
        survival_rule: Value::Singles(vec![2, 6, 9]),
        birth_rule: Value::Singles(vec![4, 6, 8, 9, 10]),
        states: 10,
        colour_method: ColourMethod::DistanceToCenter(Color::RED, Color::YELLOW),
        neighbour_method: NeighbourMethod::Moore,

        // VonNeuman pyramid
        // survival_rule: Value::Range(0..=6),
        // birth_rule: Value:::Singles(vec![1, 3]),
        // states: 2,
        // colour_method: ColourMethod::DistanceToCenter(Color::BLUE, Color::GREEN),
        // neighbour_method: NeighbourMethod::VonNeuman

        // fancy pattern
        // survival_rule: Value::Singles(vec![0, 1, 2, 3, 7, 8, 9, 11, 13, 18, 21, 22, 24, 26]),
        // birth_rule: Value::Singles(vec![4, 13, 17, 20, 21, 22, 23, 24, 26]),
        // states: 4,
        // colour_method: ColourMethod::StateLerp(Color::BLUE, Color::RED),
        // neighbour_method: NeighbourMethod::Moore

        // crystals
        // survival_rule: Value::Singles(vec![5, 6, 7, 8]),
        // birth_rule: Value::Singles(vec![6, 7, 9]),
        // states: 10,
        // colour_method: ColourMethod::DistanceToCenter(Color:BLUE, Color:GREEN),
        // neighbour_method: NeighbourMethod::Moore

        // swapping structures
        // survival_rule: Value::Singles(vec![3, 6, 9]),
        // birth_rule: Value::Singles(vec![4, 8, 10]),
        // states: 20,
        // colour_method: ColourMethod::StateLerp(Color::GREEN, Color:RED),
        // neighbour_method: NeighbourMethod::Moore

        // slowly expanding blob
        // survival_rule: Value::Range(9..=26),
        // birth_rule: Value::Singles(vec![5, 6, 7, 12, 13, 15]),
        // states: 20,
        // colour_method: ColourMethod::StateLerp(Color::BLUE, Color::YELLOW),
        // neighbour_method: NeighbourMethod::Moore

        // 445 rule
        // survival_rule: Value::Single(4),
        // birth_rule: Value::Single(4),
        // states: 5,
        // colour_method: ColourMethod::StateLerp(Color::RED, Color::BLACK),
        // neighbour_method: NeighbourMethod::Moore

        // expand, then die
        // survival_rule: Value::Single(4),
        // birth_rule: Value::Single(3),
        // states: 20,
        // colour_method: ColourMethod::StateLerp(Color::RED, Color::BLACK),
        // neighbour_method: NeighbourMethod::Moore

        // ???
        // survival_rule: Value::Singles(vec![6, 7]),
        // birth_rule: Value::Singles(vec![4, 6, 9, 10, 11]),
        // states: 6,
        // colour_method: ColourMethod::StateLerp(Color::RED, Color::BLUE),
        // neighbour_method: NeighbourMethod::Moore

        // large lines
        // survival_rule: Value::Single(5),
        // birth_rule: Value::Singles(vec![4, 6, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23, 24]),
        // states: 35,
        // colour_method: ColourMethod::StateLerp(Color::RED, Color::BLUE),
        // neighbour_method: NeighbourMethod::Moore
    };

    let mut task_pool_settings = DefaultTaskPoolOptions::default();

    task_pool_settings.async_compute.percent = 1.0f32;
    task_pool_settings.compute.percent = 0.0f32;
    task_pool_settings.io.percent = 0.0f32;

    App::new()
        .insert_resource(task_pool_settings)
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.65f32, 0.9f32, 0.96f32)))
        .add_event::<CellStatesChangedEvent>()
        .add_plugin(RotatingCameraPlugin)
        .add_plugin(CellMaterialPlugin)
        .insert_resource(rule)
        .add_plugin(CellsMultiThreadedPlugin)
        .add_startup_system(setup)
        .run();
}
