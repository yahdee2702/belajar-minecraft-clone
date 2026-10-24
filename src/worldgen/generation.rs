use bevy::prelude::*;

use crate::worldgen::chunks::{
    Chunk, ChunkEntities, ChunkInfo, ChunkManager, ChunkPos, PendingChunks,
};

const SIMULATION_SIZE: i32 = 8;
const RENDERING_SIZE: i32 = 6;

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    chunk_info: Res<ChunkInfo>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    let current_chunk_pos = manager.get_chunk_pos(camera.translation.as_ivec3());

    for chunk_x in -SIMULATION_SIZE..=SIMULATION_SIZE {
        for chunk_z in -SIMULATION_SIZE..=SIMULATION_SIZE {
            let pos = ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.z);

            if manager.exists(&pos) {
                continue;
            }

            let mut chunk = Chunk::new();
            chunk.perlin_generation(&chunk_info.perlin, pos);

            manager.insert(pos, chunk);
        }
    }
}

pub fn begin_rendering_chunks(
    mut pending_chunks: ResMut<PendingChunks>,
    manager: Res<ChunkManager>,
    chunk_entities: ResMut<ChunkEntities>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    let current_chunk_pos = manager.get_chunk_pos(camera.translation.as_ivec3());

    for chunk_x in -RENDERING_SIZE..=RENDERING_SIZE {
        for chunk_z in -RENDERING_SIZE..=RENDERING_SIZE {
            let pos = ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.z);

            if chunk_entities.chunks.contains_key(&pos)
                || pending_chunks.in_queue(pos)
                || !manager.exists(&pos)
            {
                continue;
            }

            pending_chunks.enqueue(pos);
        }
    }
}

pub fn remove_out_of_range_chunks(
    mut commands: Commands,
    mut chunk_entities: ResMut<ChunkEntities>,
    pending_chunks: Res<PendingChunks>,
    manager: ResMut<ChunkManager>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    let current_chunk_pos = manager.get_chunk_pos(camera.translation.as_ivec3());

    let out_of_range: Vec<ChunkPos> = manager
        .iter_out_of_range(current_chunk_pos, SIMULATION_SIZE + 2)
        .map(|(pos, _)| *pos)
        .collect();

    for chunk_pos in out_of_range {
        if pending_chunks.in_queue(chunk_pos) {
            continue;
        }

        if let Some(entity) = chunk_entities.chunks.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }
    }
}
