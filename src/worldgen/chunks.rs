use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};
use noise::{Fbm, NoiseFn, Perlin};

use crate::worldgen::{models::Faces, registry::AIR_BLOCK, rendering::FACE_NEIGHBORS};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 128;
const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn new(x: i32, z: i32) -> ChunkPos {
        ChunkPos { x, z }
    }

    pub fn from_ivec2(vec: IVec2) -> ChunkPos {
        ChunkPos::new(vec.x, vec.y)
    }

    pub fn as_ivec2(&self) -> IVec2 {
        IVec2::new(self.x, self.z)
    }

    pub fn distance_squared(&self, from: ChunkPos) -> i32 {
        self.as_ivec2().distance_squared(from.as_ivec2())
    }
}

#[derive(Resource, Debug)]
pub struct ChunkInfo {
    pub seed: u32,
    pub base_perlin: Fbm<Perlin>,
    pub mountain_perlin: Fbm<Perlin>,
    pub forest_perlin: Fbm<Perlin>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
    chunks_pending_blocks: HashMap<ChunkPos, Vec<(u8, UVec3)>>,
}

impl ChunkManager {
    pub fn insert(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    pub fn get(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    pub fn exists(&self, pos: &ChunkPos) -> bool {
        self.chunks.contains_key(pos)
    }

    pub fn get_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    pub fn get_from_vec(&self, pos: IVec2) -> Option<&Chunk> {
        self.chunks.get(&ChunkPos::new(pos.x, pos.y))
    }

    pub fn get_from_xz(&self, x: i32, z: i32) -> Option<&Chunk> {
        self.get_from_vec(IVec2::new(x, z))
    }

    pub fn get_block(&self, chunk_pos: &ChunkPos, local_position: UVec3) -> Option<u8> {
        let Some(chunk) = self.chunks.get(chunk_pos) else {
            return None;
        };

        chunk.get(
            local_position.x as usize,
            local_position.y as usize,
            local_position.z as usize,
        )
    }

    pub fn get_block_from_vec(&self, chunk_pos: IVec2, local_position: UVec3) -> Option<u8> {
        self.get_block(&ChunkPos::new(chunk_pos.x, chunk_pos.y), local_position)
    }

    pub fn get_block_by_world_pos(&self, world_pos: IVec3) -> Option<u8> {
        let chunk_pos = self.get_chunk_pos(world_pos);
        let local_pos = self.get_local_pos(world_pos);

        self.get_block(&chunk_pos, local_pos)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.chunks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ChunkPos, &mut Chunk)> {
        self.chunks.iter_mut()
    }

    pub fn iter_out_of_range(
        &self,
        chunk_pos: ChunkPos,
        range: i32,
    ) -> impl Iterator<Item = (&ChunkPos, &Chunk)> {
        self.iter()
            .filter(move |(pos, _chunk)| pos.distance_squared(chunk_pos) > (range * range))
            .into_iter()
    }

    pub fn get_chunk_pos_in_vec(&self, position: IVec3) -> IVec2 {
        let chunk_x = position.x.div_euclid(CHUNK_SIZE as i32);
        let chunk_z = position.z.div_euclid(CHUNK_SIZE as i32);

        IVec2::new(chunk_x, chunk_z)
    }

    pub fn get_chunk_pos(&self, position: IVec3) -> ChunkPos {
        let chunk_position = self.get_chunk_pos_in_vec(position);
        ChunkPos::from_ivec2(chunk_position)
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

    pub fn get_neighbor(
        &self,
        chunk_pos: &ChunkPos,
        target_position: IVec3,
    ) -> (u8, Option<ChunkPos>) {
        if target_position.y < 0 || target_position.y >= (CHUNK_HEIGHT as i32) {
            return (AIR_BLOCK, None);
        }

        let size = CHUNK_SIZE as i32;
        let world_x = chunk_pos.x * size + target_position.x;
        let world_z = chunk_pos.z * size + target_position.z;

        let chunk_x = world_x.div_euclid(size);
        let chunk_z = world_z.div_euclid(size);

        let local_x = world_x.rem_euclid(size) as usize;
        let local_z = world_z.rem_euclid(size) as usize;

        let Some(chunk) = self.get(&chunk_pos) else {
            return (AIR_BLOCK, None);
        };

        let target_chunk = if chunk_x == chunk_pos.x && chunk_z == chunk_pos.z {
            chunk
        } else {
            match self.get_from_xz(chunk_x, chunk_z) {
                Some(neighbor_chunk) => neighbor_chunk,
                None => return (AIR_BLOCK, None),
            }
        };

        (
            target_chunk
                .get(local_x, target_position.y as usize, local_z)
                .unwrap_or(AIR_BLOCK),
            Some(ChunkPos::new(chunk_x, chunk_z)),
        )
    }

    pub fn set_block(&mut self, chunk_pos: &ChunkPos, local_position: UVec3, block: u8) {
        let Some(chunk) = self.get_mut(chunk_pos) else {
            return;
        };

        chunk.set(local_position, block);
    }

    pub fn set_block_by_world_pos(&mut self, world_pos: IVec3, block: u8) {
        let chunk_pos = self.get_chunk_pos(world_pos);
        let local_pos = self.get_local_pos(world_pos);

        let Some(chunk) = self.get_mut(&chunk_pos) else {
            self.insert_pending_blocks(chunk_pos, local_pos, block);
            return;
        };

        chunk.set(local_pos, block);
    }

    pub fn insert_pending_blocks(&mut self, chunk_pos: ChunkPos, local_pos: UVec3, block: u8) {
        if self.exists(&chunk_pos) {
            return;
        }

        let block_list = self.chunks_pending_blocks.entry(chunk_pos).or_default();

        block_list.push((block, local_pos));
    }

    pub fn fill_pending_blocks(&mut self, chunk_pos: ChunkPos, chunk: &mut Chunk) {
        let Some(block_list) = self.chunks_pending_blocks.get_mut(&chunk_pos) else {
            return;
        };

        block_list.retain(|(block, local_pos)| {
            chunk.set(*local_pos, *block);

            true
        });

        self.chunks_pending_blocks.remove(&chunk_pos);
    }

    pub fn update_chunk_by_world_pos(&self, world_pos: IVec3, pending_chunks: &mut PendingChunks) {
        let chunk_pos = self.get_chunk_pos(world_pos);

        pending_chunks.enqueue(chunk_pos);
    }

    pub fn update_neighbor_chunks_by_world_pos(
        &self,
        world_pos: IVec3,
        pending_chunks: &mut PendingChunks,
    ) {
        let chunk_pos = self.get_chunk_pos(world_pos);
        let local_pos = self.get_local_pos(world_pos);

        if local_pos.x != 0 && local_pos.x != 15 && local_pos.z != 0 && local_pos.z != 15 {
            return;
        }

        for face in Faces::SIDES {
            let [dx, dy, dz] = FACE_NEIGHBORS[face as usize];
            if let (neighbor_block, Some(neighbor_chunk_pos)) = self.get_neighbor(
                &chunk_pos,
                IVec3::new(
                    local_pos.x as i32 + dx,
                    local_pos.y as i32 + dy,
                    local_pos.z as i32 + dz,
                ),
            ) {
                if neighbor_chunk_pos != chunk_pos && neighbor_block != AIR_BLOCK {
                    pending_chunks.enqueue(neighbor_chunk_pos);
                }
            }
        }
    }

    // Old Raycast (I made it myself)
    pub fn raycast(&self, current: Vec3, direction: Vec3, distance: f32) -> (u8, UVec3, ChunkPos) {
        let current_offseted = current - Vec3::splat(0.5);
        let mut cur_dist = 0.0;
        let mut block: u8 = AIR_BLOCK;
        let mut chunk_pos: IVec2 = IVec2::default();
        let mut local_pos: UVec3 = UVec3::default();

        while block == AIR_BLOCK && cur_dist < distance {
            let pos = (current_offseted + direction * cur_dist as f32)
                .ceil()
                .as_ivec3();

            chunk_pos = self.get_chunk_pos_in_vec(pos);
            local_pos = self.get_local_pos(pos);

            block = self
                .get_block_from_vec(chunk_pos, local_pos)
                .unwrap_or(AIR_BLOCK);

            cur_dist += 1.0;
        }

        (block, local_pos, ChunkPos::new(chunk_pos.x, chunk_pos.y))
    }

    // 3D DDA (AI Made - still trying to understand it)
    pub fn raycast_3d_dda(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(u8, IVec3, IVec3)> {
        if max_distance < 0.0 || !max_distance.is_finite() {
            return None;
        }

        let direction = direction.normalize_or_zero();

        if direction == Vec3::ZERO {
            return None;
        }

        // Shift the ray so integer grid cells match blocks centered
        // on integer world coordinates.
        let shifted_origin = origin + Vec3::splat(0.5);

        let mut voxel = shifted_origin.floor().as_ivec3();

        let step = IVec3::new(
            if direction.x > 0.0 {
                1
            } else if direction.x < 0.0 {
                -1
            } else {
                0
            },
            if direction.y > 0.0 {
                1
            } else if direction.y < 0.0 {
                -1
            } else {
                0
            },
            if direction.z > 0.0 {
                1
            } else if direction.z < 0.0 {
                -1
            } else {
                0
            },
        );

        let t_delta = Vec3::new(
            if direction.x != 0.0 {
                (1.0 / direction.x).abs()
            } else {
                f32::INFINITY
            },
            if direction.y != 0.0 {
                (1.0 / direction.y).abs()
            } else {
                f32::INFINITY
            },
            if direction.z != 0.0 {
                (1.0 / direction.z).abs()
            } else {
                f32::INFINITY
            },
        );

        let mut t_max = Vec3::new(
            if step.x > 0 {
                ((voxel.x + 1) as f32 - shifted_origin.x) / direction.x
            } else if step.x < 0 {
                (voxel.x as f32 - shifted_origin.x) / direction.x
            } else {
                f32::INFINITY
            },
            if step.y > 0 {
                ((voxel.y + 1) as f32 - shifted_origin.y) / direction.y
            } else if step.y < 0 {
                (voxel.y as f32 - shifted_origin.y) / direction.y
            } else {
                f32::INFINITY
            },
            if step.z > 0 {
                ((voxel.z + 1) as f32 - shifted_origin.z) / direction.z
            } else if step.z < 0 {
                (voxel.z as f32 - shifted_origin.z) / direction.z
            } else {
                f32::INFINITY
            },
        );

        let mut distance = 0.0;
        let mut face_normal = IVec3::ZERO;

        loop {
            if distance > max_distance {
                return None;
            }

            // Convert world voxel coordinates into chunk and local coordinates.

            // Skip blocks outside the vertical world bounds.
            if voxel.y >= 0 && voxel.y < CHUNK_HEIGHT as i32 {
                let chunk_pos = self.get_chunk_pos(voxel);
                let local_pos = self.get_local_pos(voxel);

                let block = self.get_block(&chunk_pos, local_pos).unwrap_or(AIR_BLOCK);

                if block != AIR_BLOCK {
                    return Some((block, voxel, face_normal));
                }
            }

            // Advance to the next voxel boundary.
            if t_max.x <= t_max.y && t_max.x <= t_max.z {
                distance = t_max.x;
                voxel.x += step.x;
                t_max.x += t_delta.x;
                face_normal = IVec3::new(-step.x, 0, 0);
            } else if t_max.y <= t_max.z {
                distance = t_max.y;
                voxel.y += step.y;
                t_max.y += t_delta.y;
                face_normal = IVec3::new(0, -step.y, 0);
            } else {
                distance = t_max.z;
                voxel.z += step.z;
                t_max.z += t_delta.z;
                face_normal = IVec3::new(0, 0, -step.z);
            }
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct ChunkEntities {
    pub chunks: HashMap<ChunkPos, Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct PendingChunks {
    chunks: VecDeque<ChunkPos>,
}

impl PendingChunks {
    pub fn enqueue(&mut self, chunk_position: ChunkPos) {
        self.chunks.push_front(chunk_position);
    }

    pub fn back_enqueue(&mut self, chunk_position: ChunkPos) {
        self.chunks.push_back(chunk_position);
    }

    pub fn in_queue(&self, chunk_position: ChunkPos) -> bool {
        self.chunks.contains(&chunk_position)
    }

    pub fn pop_next(&mut self) -> Option<ChunkPos> {
        self.chunks.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    blocks: Vec<u8>,
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
        let index = Chunk::index(
            (position.x as usize).clamp(0, CHUNK_SIZE),
            (position.y as usize).clamp(0, CHUNK_HEIGHT),
            (position.z as usize).clamp(0, CHUNK_SIZE),
        );
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
        self.blocks.get(Chunk::index(x, y, z)).cloned()
    }

    pub fn perlin_generation(
        &mut self,
        base_perlin: &Fbm<Perlin>,
        mountain_perlin: &Fbm<Perlin>,
        pos: ChunkPos,
    ) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_x = pos.x * CHUNK_SIZE as i32 + x as i32;
                let world_z = pos.z * CHUNK_SIZE as i32 + z as i32;

                let base = base_perlin.get([world_x as f64, world_z as f64]);
                let peak = mountain_perlin.get([world_x as f64, world_z as f64]);

                let height = (35.0 + (base * 10.0) + (peak * 24.0))
                    .floor()
                    .clamp(0.0, CHUNK_HEIGHT as f64);

                for y in 0..height as usize {
                    let percentage = y as f64 / (height - 1.0);

                    if percentage >= 1.0 {
                        self.set(UVec3::new(x as u32, y as u32, z as u32), 1);
                    } else if percentage >= 0.7 {
                        self.set(UVec3::new(x as u32, y as u32, z as u32), 2);
                    } else {
                        self.set(UVec3::new(x as u32, y as u32, z as u32), 3);
                    }
                }
            }
        }
    }
}
