use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    mesh::Indices,
    platform::collections::HashMap,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};

use crate::worldgen::{
    chunks::{CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkEntities, ChunkManager, ChunkPos, PendingChunks}, models::{BlockModelsManager, CullingMode, Faces, SharedBlockModelsManager}, registry::{AIR_BLOCK, RegistryManager, SharedRegistryManager}, textures::BlockAtlas,
};
pub struct ChunkMeshAttributes {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub colors: Vec<[f32; 4]>,
}

#[derive(Resource, Default)]
pub struct PendingChunksMesh {
    chunks: HashMap<ChunkPos, Task<ChunkMeshAttributes>>,
}

impl PendingChunksMesh {
    pub fn insert(&mut self, chunk_pos: ChunkPos, task: Task<ChunkMeshAttributes>) {
        self.chunks.insert(chunk_pos, task);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&ChunkPos, &mut Task<ChunkMeshAttributes>) -> bool,
    {
        self.chunks.retain(f);
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

pub fn render_pending_chunks(
    chunk_manager: Res<ChunkManager>,
    mut pending_chunks: ResMut<PendingChunks>,
    mut pending_chunk_mesh: ResMut<PendingChunksMesh>,
    registry: Res<SharedRegistryManager>,

    block_atlas: Res<BlockAtlas>,
    models_manager: Res<SharedBlockModelsManager>,
) {
    if pending_chunks.is_empty() {
        return;
    }

    let Some(chunk_pos) = pending_chunks.pop_next() else {
        return;
    };

    if chunk_manager.get(&chunk_pos).is_none() {
        return;
    };

    let task_pool = AsyncComputeTaskPool::get();

    let atlas_size = block_atlas.size;
    
    let registry     = Arc::clone(&registry.registry_manager);
    let models_manager = Arc::clone(&models_manager.models_manager);
    let cloned_models_manager = (*models_manager).clone();

    // TODO: NEED to find a better way for this
    let cloned_chunk_manager = (*chunk_manager).clone();

    let task: Task<ChunkMeshAttributes> = task_pool.spawn(async move {        
        let chunk = cloned_chunk_manager.get(&chunk_pos).unwrap();

        render_chunk(
            chunk,
            chunk_pos,
            atlas_size,
            &registry,
            &cloned_models_manager,
            &cloned_chunk_manager,
        )
    });

    pending_chunk_mesh.insert(chunk_pos, task);
}

pub fn render_chunk(
    chunk: &Chunk,
    chunk_pos: ChunkPos,
    atlas_size: UVec2,
    registry: &RegistryManager,
    models_manager: &BlockModelsManager,
    chunk_manager: &ChunkManager,
) -> ChunkMeshAttributes {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    for y in 0..CHUNK_HEIGHT {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let block = chunk.get(x, y, z).unwrap().clone();

                if block == AIR_BLOCK {
                    continue;
                }

                let Some(registered_block) = registry.get(block) else {
                    continue;
                };

                let Some(block_model) = models_manager.get(&registered_block.name) else {
                    continue;
                };

                for face in Faces::ALL {
                    let [dx, dy, dz] = FACE_NEIGHBORS[face as usize];
                    let face_texture = block_model.get_face_textures(face);

                    let culling_mode = face_texture.culling.mode;

                    let (neighbor_block, _neighbor_chunk_pos) = chunk_manager.get_neighbor(
                        &chunk_pos,
                        IVec3::new(dx + x as i32, dy + y as i32, dz + z as i32),
                    );

                    let Some(registered_block_neighbor) = registry.get(neighbor_block) else {
                        continue;
                    };

                    let Some(block_model_neighbor) =
                        models_manager.get(&registered_block_neighbor.name)
                    else {
                        continue;
                    };

                    let face_texture = block_model_neighbor.get_opposite_face_textures(face);

                    let neighbor_culling_mode = face_texture.culling.mode;

                    if neighbor_block != AIR_BLOCK {
                        let same_block = neighbor_block == block;

                        let should_cull = match (culling_mode, neighbor_culling_mode) {
                            (CullingMode::None, _) | (_, CullingMode::None) => false,

                            (CullingMode::SameBlock, _) | (_, CullingMode::SameBlock) => same_block,

                            (CullingMode::Default, CullingMode::Default) => true,
                        };

                        if should_cull {
                            continue;
                        }
                    }

                    let vertex_offset = positions.len() as u32;

                    for vertex in FACE_POSITIONS[face as usize] {
                        positions.push([
                            vertex[0] + x as f32,
                            vertex[1] + y as f32,
                            vertex[2] + z as f32,
                        ]);
                    }

                    let tint = if registered_block.name == "grass" && face == Faces::Top {
                        [0.3, 1.0, 0.2, 1.0]
                    } else if registered_block.name == "leave" {
                        [0.271, 0.42, 0.208, 1.0]
                    } else {
                        [1.0, 1.0, 1.0, 1.0]
                    };

                    colors.extend([tint; 4]);

                    normals.extend([FACE_NORMALS[face as usize]; 4]);
                    uvs.extend(block_model.get_uvs(face, atlas_size));

                    for index in FACE_INDICES[face as usize] {
                        indices.push(vertex_offset + index);
                    }
                }
            }
        }
    }

    ChunkMeshAttributes { positions, normals, uvs, indices, colors }
}

pub fn render_pending_chunks_mesh(
    mut commands: Commands,
    mut pending_chunks_mesh: ResMut<PendingChunksMesh>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    block_atlas: Res<BlockAtlas>,
) {
    if pending_chunks_mesh.is_empty() {
        return;
    }

    pending_chunks_mesh.retain(|chunk_pos, task| {
        let status = block_on(future::poll_once(task));

        let retain = status.is_none();

        let Some(chunk_data) = status else {
            return retain;
        };

        if let Some(prev_entity) = chunk_entities.chunks.get(chunk_pos) {
            commands.entity(*prev_entity).despawn();
            chunk_entities.chunks.remove(chunk_pos);
        }

        let mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, chunk_data.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, chunk_data.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, chunk_data.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, chunk_data.colors)
        .with_inserted_indices(Indices::U32(chunk_data.indices));

        let chunk_material = StandardMaterial {
            // base_color: Color::WHITE,
            base_color_texture: Some(block_atlas.handle.clone()),
            alpha_mode: AlphaMode::Mask(0.5),
            ..default()
        };

        let chunk_entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(chunk_material)),
                Transform::from_xyz(
                    (chunk_pos.x * CHUNK_SIZE as i32) as f32,
                    0.0,
                    (chunk_pos.z * CHUNK_SIZE as i32) as f32,
                ),
            ))
            .id();

        chunk_entities
            .chunks
            .insert(chunk_pos.clone(), chunk_entity);

        retain
    });
}

const FACE_POSITIONS: [[[f32; 3]; 4]; 6] = [
    // top (facing towards +y)
    [
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
    ],
    // bottom   (-y)
    [
        [-0.5, -0.5, -0.5],
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
    ],
    // right    (+x)
    [
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
    ],
    // left     (-x)
    [
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
    ],
    // back     (+z)
    [
        [-0.5, -0.5, 0.5],
        [-0.5, 0.5, 0.5],
        [0.5, 0.5, 0.5],
        [0.5, -0.5, 0.5],
    ],
    // forward  (-z)
    [
        [-0.5, -0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
    ],
];

const FACE_NORMALS: [[f32; 3]; 6] = [
    [0.0, 1.0, 0.0],
    [0.0, -1.0, 0.0],
    [1.0, 0.0, 0.0],
    [-1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0],
    [0.0, 0.0, -1.0],
];

pub const FACE_NEIGHBORS: [[i32; 3]; 6] = [
    [0, 1, 0],
    [0, -1, 0],
    [1, 0, 0],
    [-1, 0, 0],
    [0, 0, 1],
    [0, 0, -1],
];

const FACE_INDICES: [[u32; 6]; 6] = [
    [0, 3, 1, 1, 3, 2],
    [0, 1, 3, 1, 2, 3],
    [0, 3, 1, 1, 3, 2],
    [0, 1, 3, 1, 2, 3],
    [0, 3, 1, 1, 3, 2],
    [0, 1, 3, 1, 2, 3],
];
