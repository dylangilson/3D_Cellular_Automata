/*
 * Dylan Gilson
 * dylan.gilson@outlook.com
 * January 20, 2023
 */

use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Component)]
pub struct FlyCamera {
    pub acceleration: f32, // speed the camera accelerates at, defaults to 1.5
    pub max_speed: f32, // defaults to 0.5
    pub sensitivity: f32, // sensitivity of movement of camera's motion, defaults to 3.0
    pub friction: f32, // amount of deceleration to apply to camera's motion, defaults to 1.0
    pub pitch: f32, // value is always up-to-date, enforced by FlyCameraPlugin
    pub yaw: f32, // value is always up-to-date, enforced by FlyCameraPlugin
    pub velocity: Vec3, // value is always up-to-date, enforced by FlyCameraPlugin
    pub key_forward: KeyCode, // defaults to W
    pub key_left: KeyCode, // defaults to A
    pub key_backward: KeyCode, // defaults to S
    pub key_right: KeyCode, // defaults to D
    pub key_up: KeyCode, // defaults to Space
    pub key_down: KeyCode, // defaults to LShift
    pub enabled: bool // false -> disable keyboard control of camera ; defaults to true
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            acceleration: 1.5,
            max_speed: 0.5,
            sensitivity: 3.0,
            friction: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            key_forward: KeyCode::W,
            key_left: KeyCode::A,
            key_backward: KeyCode::S,
            key_right: KeyCode::D,
            key_up: KeyCode::Space,
            key_down: KeyCode::LShift,
            enabled: true
        }
    }
}

pub fn movement_axis(input: &Res<Input<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;

    if input.pressed(plus) {
        axis += 1.0;
    }

    if input.pressed(minus) {
        axis -= 1.0;
    }

    axis
}

fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    let f_flattened = Vec3::new(f.x, f.y, f.z).normalize();

    f_flattened
}

// rotate vector 90 degrees to get strafe direction
fn strafe_vector(rotation: &Quat) -> Vec3 {
    Quat::from_rotation_y(90.0f32.to_radians()).mul_vec3(forward_walk_vector(rotation)).normalize()
}

fn camera_movement_system(time: Res<Time>, keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut FlyCamera, &mut Transform)>) {
    for (mut options, mut transform) in query.iter_mut() {
        let (axis_h, axis_v, axis_float) = if options.enabled {
            (
                movement_axis(&keyboard_input, options.key_right, options.key_left),
                movement_axis(&keyboard_input, options.key_backward, options.key_forward),
                movement_axis(&keyboard_input, options.key_up, options.key_down)
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        let rotation = transform.rotation;

        let acceleration: Vec3 = (strafe_vector(&rotation) * axis_h) + (forward_walk_vector(&rotation) * axis_v) + (Vec3::Y * axis_float);
        let acceleration: Vec3 = if acceleration.length() != 0.0 {
            acceleration.normalize() * options.acceleration
        } else {
            Vec3::ZERO
        };

        let friction: Vec3 = if options.velocity.length() != 0.0 {
            options.velocity.normalize() * -1.0 * options.friction
        } else {
            Vec3::ZERO
        };

        options.velocity += acceleration * time.delta_seconds();

        // clamp within max_speed
        if options.velocity.length() > options.max_speed {
            options.velocity = options.velocity.normalize() * options.max_speed;
        }

        let delta_friction = friction * time.delta_seconds();

        options.velocity = if (options.velocity + delta_friction).signum() != options.velocity.signum() {
            Vec3::ZERO
        } else {
            options.velocity + delta_friction
        };

        transform.translation += options.velocity;
    }
}

fn mouse_motion_system(time: Res<Time>, mut mouse_event_reader: EventReader<MouseMotion>, mut query: Query<(&mut FlyCamera, &mut Transform)>) {
    let mut delta: Vec2 = Vec2::ZERO;

    for event in mouse_event_reader.iter() {
        delta += event.delta;
    }

    if delta.is_nan() {
        return;
    }

    for (mut options, mut transform) in query.iter_mut() {
        if !options.enabled {
            continue;
        }

        options.yaw -= delta.x * options.sensitivity * time.delta_seconds();
        options.pitch += delta.y * options.sensitivity * time.delta_seconds();
        options.pitch = options.pitch.clamp(-89.0, 89.0);

        let yaw_radians = options.yaw.to_radians();
        let pitch_radians = options.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_movement_system).add_system(mouse_motion_system);
    }
}
