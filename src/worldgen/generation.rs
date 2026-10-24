use std::time::Instant;

use bevy::prelude::*;
use noise::Perlin;

use crate::{GameState, worldgen::chunks::{Chunk, ChunkManager, ChunkPos, PendingChunks}};

pub fn generate_chunks(
    mut manager: ResMut<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let start = Instant::now();
    let seed = rand::random_range(1..1000000);
    let perlin = Perlin::new(seed);

    let chunk_width = 4;

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

    next_state.set(GameState::BeforePlaying);
    let elapsed = start.elapsed();
    println!("Chunk terrain generation: {elapsed:?}");
}