use bevy::prelude::*;

use crate::{GameState, worldgen::chunks::{Chunk, ChunkManager, ChunkPos}};

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let default_chunk = Chunk::new()
        .with_fill(UVec3::new(0, 0, 0), UVec3::new(15,1,15), 2)
        .with_fill(UVec3::new(0, 2, 0), UVec3::new(15,2,15), 1);
    manager.insert(
        ChunkPos(IVec2::new(0, 0)),
        default_chunk.clone(),
    );

    manager.insert(
        ChunkPos(IVec2::new(0, 1)),
        default_chunk.clone(),
    );
    
    manager.insert(
        ChunkPos(IVec2::new(1, 0)),
        default_chunk.clone(),
    );

    manager.insert(
        ChunkPos(IVec2::new(2, 1)),
        default_chunk.clone(),
    );

    next_state.set(GameState::RenderingChunk);
}