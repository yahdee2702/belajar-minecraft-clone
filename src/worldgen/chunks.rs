use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::worldgen::registry::AIR_BLOCK;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> ChunkPos {
        ChunkPos {
            x, z,
        }
    }
}

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

    pub fn get_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    pub fn get_from_vec(&self, pos: IVec2) -> Option<&Chunk> {
        self.chunks.get(&ChunkPos::new(pos.x, pos.y))
    }

    pub fn get_from_xz(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.get_from_vec(IVec2::new(x,z))
    }

    pub fn get_block(&self, chunk_pos: ChunkPos, local_position: UVec3) -> Option<u8> {
        let Some(chunk) = self.chunks.get(&chunk_pos) else {
            return None;
        };

        chunk.get(local_position.x as usize, local_position.y as usize, local_position.z as usize)
    }

    pub fn get_block_from_vec(&self, chunk_pos: IVec2, local_position: UVec3) -> Option<u8> {
        self.get_block(ChunkPos::new(chunk_pos.x, chunk_pos.y), local_position)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.chunks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ChunkPos, &mut Chunk)> {
        self.chunks.iter_mut()
    }
    
    pub fn get_chunk_pos(&self, position: IVec3) -> IVec2 {
        let chunk_x = position.x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = position.z.div_euclid(CHUNK_SIZE as i32);

        IVec2::new(chunk_x, chunk_z)
    }

    pub fn get_local_pos(&self, position: IVec3) -> UVec3 {
        let local_x = position.x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_z = position.z.rem_euclid(CHUNK_SIZE as i32) as usize;
        
        UVec3::new(
            local_x as u32, 
            position.y.clamp(0, CHUNK_HEIGHT as i32) as u32, 
            local_z as u32,
        )
    }
    
    pub fn raycast(&self, current: Vec3, direction: Vec3, distance: i32) -> (u8, UVec3, ChunkPos) {
        let mut cur_dist = 0;
        let mut block: u8 = AIR_BLOCK;
        let mut chunk_pos: IVec2 = IVec2::default();
        let mut local_pos: UVec3 = UVec3::default();

        while block == AIR_BLOCK && cur_dist < distance {
            let pos = (current + direction * cur_dist as f32)
                .floor()
                .as_ivec3();

            chunk_pos = self.get_chunk_pos(pos);
            local_pos = self.get_local_pos(pos);
            
            block = self.get_block_from_vec(chunk_pos, local_pos).unwrap_or(AIR_BLOCK);
            
            cur_dist += 1;
        }

        (block, local_pos, ChunkPos::new(chunk_pos.x, chunk_pos.y))
    }
}

#[derive(Resource, Debug, Default)]
pub struct ChunkEntities {
    pub chunks: HashMap<ChunkPos, Entity>
}

#[derive(Resource, Debug, Default)]
pub struct PendingChunks {
    chunks: VecDeque<ChunkPos>
}

impl PendingChunks {
    pub fn enqueue(&mut self, chunk_position: ChunkPos) {
        self.chunks.push_front(chunk_position);
    }

    pub fn pop_next(&mut self) -> Option<ChunkPos> {
        self.chunks.pop_front()
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

    pub fn set(&mut self, position: UVec3, block: u8) {
        let index = Chunk::index(position.x as usize, position.y as usize, position.z as usize);
        self.blocks[index] = block;
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
