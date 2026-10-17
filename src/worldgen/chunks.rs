use bevy::{platform::collections::HashMap, prelude::*};

use crate::worldgen::registry::AIR_BLOCK;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos(pub IVec2);

#[derive(Resource, Debug, Default)]
pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>
}

impl ChunkManager {
    pub fn insert(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    pub fn get(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    pub fn get_from_xz(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.get(&ChunkPos(IVec2::new(x,z)))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.chunks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ChunkPos, &mut Chunk)> {
        self.chunks.iter_mut()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    blocks: Vec<u8>
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![AIR_BLOCK; CHUNK_VOLUME],
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + CHUNK_SIZE * (z + CHUNK_SIZE * y)
    }

    pub fn fill(&mut self, start: UVec3, end: UVec3, block: u8) {
        for z in start.z..=end.z {
            for y in start.y..=end.y {
                let start_index = Chunk::index(start.x as usize, y as usize, z as usize);
                let end_index = start_index + (end.x - start.x + 1) as usize;

                self.blocks[start_index..end_index].fill(block as u8);
            }
        }
    }

    pub fn with_fill(mut self, start: UVec3, end: UVec3, block: u8) -> Self {
        self.fill(start, end, block);
        self
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        self.blocks.get(Chunk::index(x,y,z)).cloned()
    }
}
