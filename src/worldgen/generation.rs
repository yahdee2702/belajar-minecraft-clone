use bevy::prelude::*;
use noise::Perlin;

use crate::{GameState, worldgen::chunks::{Chunk, ChunkManager, ChunkPos, PendingChunks}};

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let seed = 21919201;
    let perlin = Perlin::new(seed);

    let chunk_width = 8;

    for chunk_x in 0..chunk_width {
        for chunk_z in 0..chunk_width {
            let pos = ChunkPos::new(chunk_x, chunk_z);
            let mut chunk = Chunk::new();
            chunk.perlin_generation(&perlin, pos);

            manager.insert(
                pos,
                chunk,
            );

            pending_chunks.enqueue(pos);
        }
    }

    next_state.set(GameState::Playing);
}