
use bevy::prelude::*;

use crate::worldgen::chunks::{Chunk, ChunkEntities, ChunkInfo, ChunkManager, ChunkPos, PendingChunks};

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    chunk_info: Res<ChunkInfo>,
    chunk_entities: Res<ChunkEntities>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    let current_chunk_pos = manager.get_chunk_pos_in_vec(camera.translation.as_ivec3());

    let render_size = 6;
    let simulation_size = 8;

    for chunk_x in -simulation_size..simulation_size {
        for chunk_z in -simulation_size..simulation_size {
            let pos = ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.y);

            if manager.exists(&pos) {
                continue;
            }

            let mut chunk = Chunk::new();
            chunk.perlin_generation(&chunk_info.perlin, pos);

            manager.insert(
                pos,
                chunk,
            );
        }
    }

    for chunk_x in -render_size..render_size {
        for chunk_z in -render_size..render_size {
            let pos = ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.y);

            if chunk_entities.chunks.contains_key(&pos) || pending_chunks.in_queue(pos) || !manager.exists(&pos) {
                continue;
            }

            pending_chunks.enqueue(pos);
        }
    }
}