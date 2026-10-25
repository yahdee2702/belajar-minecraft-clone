use bevy::prelude::*;
use noise::NoiseFn;

use crate::worldgen::{
    chunks::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkInfo, ChunkManager, ChunkPos},
    registry::AIR_BLOCK,
};

const SIMULATION_SIZE: i32 = 8;

pub struct Structure {
    pub position: IVec3,
    pub ignore_air: bool,
    pub width: usize,
    pub height: usize,
    pub volume: usize,
    pub blocks: Vec<u8>,
}

#[derive(Resource, Debug, Default)]
pub struct LoadedStructure {
    pub chunks: Vec<ChunkPos>,
}

impl Structure {
    pub fn new(position: IVec3, width: usize, height: usize, ignore_air: bool) -> Self {
        let volume = width * height * width;
        Self {
            position: position,
            ignore_air: ignore_air,
            width: width,
            height: height,
            volume: volume,
            blocks: vec![AIR_BLOCK; volume],
        }
    }

    pub fn new_tree(position: IVec3) -> Self {
        let width = 8;
        let height = 10;
        let mut tree = Self::new(position, width, height, true);

        // Leaves
        tree.fill(UVec3::new(2, 7, 2), UVec3::new(4, 8, 4), 7);

        // Leaves
        tree.fill(UVec3::new(1, 5, 1), UVec3::new(5, 6, 5), 7);

        // Trunk
        tree.fill(UVec3::new(3, 0, 3), UVec3::new(3, 4, 3), 6);

        tree
    }

    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + self.width * (z + self.width * y)
    }

    pub fn set(&mut self, position: UVec3, block: u8) {
        if position.x as usize >= self.width
            || position.y as usize >= self.height
            || position.z as usize >= self.width
        {
            return;
        }

        let index = self.index(
            position.x as usize,
            position.y as usize,
            position.z as usize,
        );

        self.blocks[index] = block;
    }

    pub fn fill(&mut self, start: UVec3, end: UVec3, block: u8) {
        let start = start.clamp(
            UVec3::ZERO,
            UVec3::new(self.width as u32, self.height as u32, self.width as u32),
        );

        let end = end.clamp(
            UVec3::ZERO,
            UVec3::new(self.width as u32, self.height as u32, self.width as u32),
        );

        for z in start.z..=end.z {
            for y in start.y..=end.y {
                let start_index = self.index(start.x as usize, y as usize, z as usize);
                let end_index = start_index + (end.x - start.x + 1) as usize;

                self.blocks[start_index..end_index].fill(block as u8);
            }
        }
    }

    pub fn get_position(&self, index: usize) -> UVec3 {
        let x = index % self.width;
        let z = (index / self.width) % self.width;
        let y = index / (self.width * self.width);

        UVec3::new(x as u32, y as u32, z as u32)
    }

    pub fn get_world_position(&self, index: usize) -> IVec3 {
        let local_position = self.get_position(index).as_ivec3();

        let center_offset = IVec3::new(
            (self.width / 2) as i32,
            0,
            (self.width / 2) as i32,
        );

        self.position + local_position - center_offset
    }

    pub fn place(&self, chunk_manager: &mut ChunkManager) {
        for (index, block) in self.blocks.iter().enumerate() {
            if *block == AIR_BLOCK && self.ignore_air {
                continue;
            }

            let position = self.get_world_position(index);

            chunk_manager.set_block_by_world_pos(position, *block);
        }
    }
}

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
            chunk.perlin_generation(
                &chunk_info.base_perlin,
                &chunk_info.mountain_perlin,
                &chunk_info.forest_perlin,
                pos,
            );

            manager.insert(pos, chunk);
        }
    }
}

pub fn generate_structures(
    mut manager: ResMut<ChunkManager>,
    mut loaded_structure: ResMut<LoadedStructure>,
    chunk_info: Res<ChunkInfo>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    let tree_distance = 10;

    let current_chunk_pos = manager.get_chunk_pos(camera.translation.as_ivec3());

    let base_perlin = &chunk_info.base_perlin;
    let mountain_perlin = &chunk_info.mountain_perlin;
    let forest_perlin = &chunk_info.forest_perlin;

    for chunk_x in -SIMULATION_SIZE..=SIMULATION_SIZE {
        for chunk_z in -SIMULATION_SIZE..=SIMULATION_SIZE {
            let pos = ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.z);
            
            if !manager.exists(&pos) || loaded_structure.chunks.contains(&pos) {
                continue;
            }
            
            let mut existing_trees: Vec<IVec3> = Vec::new();
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let world_x = pos.x * CHUNK_SIZE as i32 + x as i32;
                    let world_z = pos.z * CHUNK_SIZE as i32 + z as i32;

                    let pos_x = world_x;
                    let pos_z = world_z;

                    let base = base_perlin.get([pos_x as f64, pos_z as f64]);
                    let peak = mountain_perlin.get([pos_x as f64, pos_z as f64]);
                    let tree_freq = forest_perlin.get([pos_x as f64, pos_z as f64]);

                    if tree_freq > 0.54 {
                        let too_close = existing_trees.iter().any(|other_pos| {
                            let dx = other_pos.x - pos_x;
                            let dz = other_pos.z - pos_z;

                            dx * dx + dz * dz < tree_distance * tree_distance
                        });

                        if too_close && !existing_trees.is_empty() {
                            continue;
                        }
                        
                        let height = (35.0 + (base * 10.0) + (peak * 24.0))
                            .floor()
                            .clamp(0.0, CHUNK_HEIGHT as f64);

                        let pos = IVec3::new(pos_x, height as i32, pos_z);

                        let tree = Structure::new_tree(pos);
                        tree.place(&mut manager);

                        existing_trees.push(pos);
                    }
                }
            }

            loaded_structure.chunks.push(pos);
        }
    }
}
