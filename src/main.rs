use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::{CursorGrabMode, CursorOptions}};

use crate::worldgen::{chunks::ChunkManager, generation::generate_chunks, models::{BlockModel, BlockModelsLoader, BlockModelsManager, PendingBlockModels, collect_loaded_models}, registry::{RegistryManager, register_blocks}, rendering::render_chunks, textures::BlockAtlas};

pub mod worldgen;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Registering,
    LoadingAsset,
    CollectingAsset,
    LoadingChunk,
    RenderingChunk,
    Playing,
}

#[derive(Resource, Debug, Default)]
pub struct CollectingProgress {
    block_models_loaded: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)

        .init_resource::<ChunkManager>()
        .init_resource::<RegistryManager>()
        .init_resource::<BlockModelsManager>()

        .init_asset::<BlockModel>()
        .init_asset_loader::<BlockModelsLoader>()

        .init_state::<GameState>()

        .add_systems(Startup, register_blocks)

        .add_systems(OnEnter(GameState::LoadingAsset), load_asset)
        .add_systems(FixedUpdate, check_asset_loaded.run_if(in_state(GameState::LoadingAsset)))

        .add_systems(OnEnter(GameState::CollectingAsset), collect_loaded_models)
        .add_systems(FixedUpdate, check_asset_collected.run_if(in_state(GameState::CollectingAsset)))

        .add_systems(OnEnter(GameState::LoadingChunk), generate_chunks)
        .add_systems(OnEnter(GameState::RenderingChunk), render_chunks)
        .add_systems(OnEnter(GameState::Playing), setup_scene)
        .add_systems(Update, (grab_mouse, camera_movement_key).run_if(in_state(GameState::Playing)))
        .run();
}

fn load_asset(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<RegistryManager>,
) {
    commands.insert_resource(BlockAtlas {
        handle: asset_server.load("textures/block_atlas.png"),
    });

    let mut pending_models = PendingBlockModels::default();

    for (_id, registered_block) in registry.iter() {
        let block_handle: Handle<BlockModel> = asset_server.load(registered_block.model.clone());

        pending_models.handles.push(
            (registered_block.name.clone(), block_handle)
        );
    }
    
    commands.insert_resource(pending_models);
}

fn check_asset_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    block_atlas: Res<BlockAtlas>,
    pending_models: Res<PendingBlockModels>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if asset_server.is_loaded_with_dependencies(block_atlas.handle.id())
        && pending_models.handles.iter().all(|handle| {
            asset_server.is_loaded_with_dependencies(handle.1.id())
        }) {

        commands.insert_resource(CollectingProgress::default());
        next_state.set(GameState::CollectingAsset);
    }
}

fn check_asset_collected(
    progress: Res<CollectingProgress>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if progress.block_models_loaded {
        next_state.set(GameState::LoadingChunk);
    }
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
        Text::new("test text"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        }
    ));
}

fn grab_mouse(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn camera_movement_key(
    time: Res<Time>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Transform, With<Camera3d>>
) {
    let mut transform = camera.into_inner();
    let speed = 15.0;
    let sensitivity = 0.002;

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

    let delta = mouse_motion.delta;
    transform.rotate_y(-delta.x * sensitivity);
    transform.rotate_local_x(-delta.y * sensitivity);
}