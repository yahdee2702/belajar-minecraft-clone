use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

use crate::worldgen::{
    chunks::{ChunkEntities, ChunkManager, ChunkPos, PendingChunks}, generation::generate_chunks, models::{
        BlockModel, BlockModelsLoader, BlockModelsManager, Faces, PendingBlockModels,
        collect_loaded_models,
    }, registry::{AIR_BLOCK, RegistryManager, register_blocks}, rendering::{FACE_NEIGHBORS, render_pending_chunks}, textures::BlockAtlas,
};

pub mod worldgen;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Registering,
    LoadingAsset,
    CollectingAsset,
    LoadingChunk,
    Playing,
}

#[derive(Resource, Debug, Default)]
pub struct CollectingProgress {
    block_models_loaded: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<ChunkManager>()
        .init_resource::<PendingChunks>()
        .init_resource::<ChunkEntities>()
        .init_resource::<RegistryManager>()
        .init_resource::<BlockModelsManager>()
        .init_asset::<BlockModel>()
        .init_asset_loader::<BlockModelsLoader>()
        .init_state::<GameState>()
        .add_systems(Startup, register_blocks)
        .add_systems(Update, update_fps_counter)
        .add_systems(OnEnter(GameState::LoadingAsset), load_asset)
        .add_systems(
            FixedUpdate,
            check_asset_loaded.run_if(in_state(GameState::LoadingAsset)),
        )
        .add_systems(OnEnter(GameState::CollectingAsset), collect_loaded_models)
        .add_systems(
            FixedUpdate,
            check_asset_collected.run_if(in_state(GameState::CollectingAsset)),
        )
        .add_systems(OnEnter(GameState::LoadingChunk), generate_chunks)
        .add_systems(OnEnter(GameState::Playing), setup_scene)
        .add_systems(
            Update,
            (
                grab_mouse,
                camera_movement_key,
                raycast_test,
                raycast_test2,
                render_pending_chunks,
            )
                .run_if(in_state(GameState::Playing)),
        )
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

        pending_models
            .handles
            .push((registered_block.name.clone(), block_handle));
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
        && pending_models
            .handles
            .iter()
            .all(|handle| asset_server.is_loaded_with_dependencies(handle.1.id()))
    {
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

#[derive(Component)]
struct NormalText;

#[derive(Component)]
struct FpsText;

fn setup_scene(mut commands: Commands) {
    // Circle Table Mesh

    // Directional Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -45.0_f32.to_radians(),
            -30.0_f32.to_radians(),
            0.0_f32,
        )),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Atur warna latar belakang khusus untuk kamera ini
            clear_color: ClearColorConfig::Custom(Color::srgb(0.361, 0.871, 0.89)),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Text::new("Coord"),
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: percent(50),
            ..default()
        },
        NormalText,
    ));

    commands.spawn((
        Text::new("FPS: --"),
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
        FpsText,
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: px(10),
            height: px(10),
            left: percent(50),
            top: percent(50),
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

fn update_fps_counter(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diagnostic| diagnostic.smoothed())
    else {
        return;
    };

    for mut text in &mut query {
        **text = format!("FPS: {:.0}", fps);
    }
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
    mut normal_text: Query<&mut Text, With<NormalText>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Transform, With<Camera3d>>,
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

    for mut text in normal_text.iter_mut() {
        text.0 = format!(
            "X:{0} Y:{1} Z:{2}",
            transform.translation.x, transform.translation.y, transform.translation.z,
        )
        .to_string();
    }
}

fn raycast_test(
    camera: Single<&mut Transform, With<Camera3d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    registry: Res<RegistryManager>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let current_pos = camera.translation;
        let direction = camera.forward().normalize_or_zero();

        let Some((block, position_hit, _face_hit)) = chunk_manager.raycast_3d_dda(current_pos, direction, 30.0) else {
            return;
        };

        let chunk_pos = ChunkPos::from_ivec2(chunk_manager.get_chunk_pos(position_hit));
        let local_pos = chunk_manager.get_local_pos(position_hit);

        let Some(registered_block) = registry.get(block) else {
            return;
        };

        println!(
            "Block ID: {0} | Block Name: {1}",
            block, registered_block.name
        );
        if block == AIR_BLOCK {
            return;
        }

        let Some(chunk) = chunk_manager.get_mut(&chunk_pos) else {
            return;
        };

        chunk.set(local_pos, AIR_BLOCK);

        for face in Faces::SIDES {
            let [dx, dy, dz] = FACE_NEIGHBORS[face as usize];
            if let (_neighbor_block, Some(neighbor_chunk_pos)) = chunk_manager.get_neighbor(
                &chunk_pos,
                IVec3::new(
                    local_pos.x as i32 + dx,
                    local_pos.y as i32 + dy,
                    local_pos.z as i32 + dz,
                ),
            ) {
                if neighbor_chunk_pos != chunk_pos {
                    pending_chunks.enqueue(neighbor_chunk_pos);
                }
            }
        }

        pending_chunks.enqueue(chunk_pos);
    }
}

fn raycast_test2(
    camera: Single<&mut Transform, With<Camera3d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    registry: Res<RegistryManager>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        let current_pos = camera.translation;
        let direction = camera.forward().normalize_or_zero();

        let Some((block, position_hit, face_hit)) = chunk_manager.raycast_3d_dda(current_pos, direction, 30.0) else {
            return;
        };

        let chunk_pos = ChunkPos::from_ivec2(chunk_manager.get_chunk_pos(position_hit + face_hit));
        let local_pos = chunk_manager.get_local_pos(position_hit + face_hit);

        let Some(registered_block) = registry.get(block) else {
            return;
        };

        println!(
            "Block ID: {0} | Block Name: {1}",
            block, registered_block.name
        );
        if block == AIR_BLOCK {
            return;
        }

        let Some(chunk) = chunk_manager.get_mut(&chunk_pos) else {
            return;
        };

        chunk.set(local_pos, 4);

        for face in Faces::SIDES {
            let [dx, dy, dz] = FACE_NEIGHBORS[face as usize];
            if let (_neighbor_block, Some(neighbor_chunk_pos)) = chunk_manager.get_neighbor(
                &chunk_pos,
                IVec3::new(
                    local_pos.x as i32 + dx,
                    local_pos.y as i32 + dy,
                    local_pos.z as i32 + dz,
                ),
            ) {
                if neighbor_chunk_pos != chunk_pos {
                    pending_chunks.enqueue(neighbor_chunk_pos);
                }
            }
        }

        pending_chunks.enqueue(chunk_pos);
    }
}
