use bevy::prelude::*;

use crate::worldgen::{chunks::ChunkManager, generation::generate_chunks, rendering::render_chunks};

pub mod worldgen;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ChunkManager>()
        .add_systems(Startup, generate_chunks)
        .add_systems(Startup, render_chunks)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, camera_movement_key)
        .run();
}

fn setup_scene(
    mut commands: Commands,
) {
    // Circle Table Mesh
    
    // Directional Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(
            Quat::from_euler(
                EulerRot::XYZ, 
                -45.0_f32.to_radians(), 
                -30.0_f32.to_radians(), 
                0.0_f32,
            )
        ),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.,0.,0.).looking_at(Vec3::ZERO, Vec3::Y)
    ));

    commands.spawn((
        Text::new("Truly usually more"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        }
    ));
}

fn camera_movement_key(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Transform, With<Camera3d>>
) {
    let mut transform = camera.into_inner();
    let speed = 15.0;

    if keyboard.pressed(KeyCode::KeyW) {
        let dir = *transform.forward();
        transform.translation += dir * speed * time.delta_secs()
    }

    if keyboard.pressed(KeyCode::KeyS) {
        let dir = *transform.back();
        transform.translation += dir * speed * time.delta_secs()
    }

    if keyboard.pressed(KeyCode::KeyA) {
        let dir = *transform.left();
        transform.translation += dir * speed * time.delta_secs()
    }

    if keyboard.pressed(KeyCode::KeyD) {
        let dir = *transform.right();
        transform.translation += dir * speed * time.delta_secs()
    }

    if keyboard.pressed(KeyCode::KeyE) {
        let dir = *transform.up();
        transform.translation += dir * speed * time.delta_secs();
    }

    if keyboard.pressed(KeyCode::KeyQ) {
        let dir = *transform.down();
        transform.translation += dir * speed * time.delta_secs();
    }

    if keyboard.pressed(KeyCode::ArrowRight) {
        transform.rotate_local_y((-10.0_f32 * speed * time.delta_secs()).to_radians());
    }

    if keyboard.pressed(KeyCode::ArrowLeft) {
        transform.rotate_local_y((10.0_f32 * speed * time.delta_secs()).to_radians());
    }
}