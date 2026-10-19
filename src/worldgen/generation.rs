use bevy::prelude::*;

use crate::{GameState, worldgen::chunks::{Chunk, ChunkManager, ChunkPos, PendingChunks}};

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let default_chunk = Chunk::new()
        .with_fill(UVec3::new(0, 0, 0), UVec3::new(15,1,15), 2)
        .with_fill(UVec3::new(0, 2, 0), UVec3::new(15,2,15), 1);
    manager.insert(
        ChunkPos::new(0, 0),
        default_chunk.clone(),
    );

    pending_chunks.enqueue(ChunkPos::new(0,0));

    manager.insert(
        ChunkPos::new(0, 1),
        default_chunk.clone(),
    );

    pending_chunks.enqueue(ChunkPos::new(0,1));
    
    manager.insert(
        ChunkPos::new(1, 0),
        default_chunk.clone(),
    );

    pending_chunks.enqueue(ChunkPos::new(1,0));

    manager.insert(
        ChunkPos::new(2, 1),
        default_chunk.clone(),
    );

    pending_chunks.enqueue(ChunkPos::new(2, 1));

    next_state.set(GameState::Playing);
}