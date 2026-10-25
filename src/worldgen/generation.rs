use bevy::{platform::collections::HashMap, prelude::*};
use noise::NoiseFn;

use crate::worldgen::{
    chunks::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkInfo, ChunkManager, ChunkPos},
    models::Faces,
    registry::AIR_BLOCK,
    rendering::FACE_NEIGHBORS,
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
    pub chunks: HashMap<ChunkPos, Vec<LoadedStructureInfo>>,
}

impl LoadedStructure {
    pub fn too_close(
        &self,
        chunk_pos: ChunkPos,
        position: IVec3,
        list: &Vec<LoadedStructureInfo>,
    ) -> bool {
        if self.is_too_close(position, list) {
            return true;
        }

        for face in Faces::SIDES {
            let [dx, _dy, dz] = FACE_NEIGHBORS[face as usize];
            let neighbor_pos = ChunkPos::new(chunk_pos.x as i32 + dx, chunk_pos.z as i32 + dz);

            let Some(list) = self.chunks.get(&neighbor_pos) else {
                continue;
            };

            if self.is_too_close(position, list) {
                return true;
            }
        }

        return false;
    }

    fn is_too_close(&self, position: IVec3, list: &Vec<LoadedStructureInfo>) -> bool {
        for loaded_structure in list {
            let other_pos: IVec3 = loaded_structure.position;
            let distance = loaded_structure.distance;

            let dx = other_pos.x - position.x;
            let dz = other_pos.z - position.z;

            if dx * dx + dz * dz < distance * distance {
                return true;
            }
        }

        return false;
    }
}

#[derive(Debug)]
pub struct LoadedStructureInfo {
    pub name: String,
    pub position: IVec3,
    pub distance: i32,
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
        let width = 6;
        let height = 10;
        let mut tree = Self::new(position, width, height, true);

        // Leaves
        tree.fill(UVec3::new(2, 7, 2), UVec3::new(4, 8, 4), 7);

        // Leaves
        tree.fill(UVec3::new(1, 5, 1), UVec3::new(5, 6, 5), 7);

        // Trunk
        tree.fill(UVec3::new(3, 0, 3), UVec3::new(3, 6, 3), 6);

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

        let center_offset = IVec3::new((self.width / 2) as i32, 0, (self.width / 2) as i32);

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
            chunk.perlin_generation(&chunk_info.base_perlin, &chunk_info.mountain_perlin, pos);

            manager.fill_pending_blocks(pos, &mut chunk);
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
    let current_chunk_pos = manager.get_chunk_pos(camera.translation.as_ivec3());

    let base_perlin = &chunk_info.base_perlin;
    let mountain_perlin = &chunk_info.mountain_perlin;
    let forest_perlin = &chunk_info.forest_perlin;

    for chunk_x in -SIMULATION_SIZE..=SIMULATION_SIZE {
        for chunk_z in -SIMULATION_SIZE..=SIMULATION_SIZE {
            let chunk_pos =
                ChunkPos::new(chunk_x + current_chunk_pos.x, chunk_z + current_chunk_pos.z);

            if !manager.exists(&chunk_pos) || loaded_structure.chunks.contains_key(&chunk_pos) {
                continue;
            }

            let mut structure_info: Vec<LoadedStructureInfo> = Vec::new();

            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
                    let world_z = chunk_pos.z * CHUNK_SIZE as i32 + z as i32;

                    let pos_x = world_x;
                    let pos_z = world_z;

                    let base = base_perlin.get([pos_x as f64, pos_z as f64]);
                    let peak = mountain_perlin.get([pos_x as f64, pos_z as f64]);
                    let tree_freq = forest_perlin.get([pos_x as f64, pos_z as f64]);

                    if tree_freq > 0.54 {
                        let height = (35.0 + (base * 10.0) + (peak * 24.0))
                            .floor()
                            .clamp(0.0, CHUNK_HEIGHT as f64);

                        let pos = IVec3::new(pos_x, height as i32, pos_z);
                        if loaded_structure.too_close(chunk_pos, pos, &structure_info) {
                            continue;
                        }

                        let tree = Structure::new_tree(pos);
                        tree.place(&mut manager);

                        structure_info.push(LoadedStructureInfo {
                            name: "tree".to_string(),
                            position: pos,
                            distance: 7,
                        });
                    }
                }
            }

            loaded_structure.chunks.insert(chunk_pos, structure_info);
        }
    }
}
