use bevy::{asset::RenderAssetUsages, mesh::Indices, prelude::*};

use crate::worldgen::chunks::{Blocks, CHUNK_HEIGHT, CHUNK_SIZE, Chunk, ChunkManager, ChunkPos};

pub fn render_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    manager: Res<ChunkManager>,
) {
    for (chunk_pos, chunk) in manager.iter() {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for y in 0..CHUNK_HEIGHT {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get(x as usize,y as usize, z as usize).unwrap().clone();
                    
                    if block == (Blocks::Air as u8) {
                        continue
                    }

                    for face in 0..6 {
                        let [dx, dy, dz] = FACE_NEIGHBORS[face];

                        let neighbor_block = get_neighbor(
                            &manager, 
                            chunk_pos, 
                            chunk, 
                            dx + x as i32, 
                            dy + y as i32, 
                            dz + z as i32,
                        );

                        if neighbor_block != Blocks::Air {
                            continue;
                        }

                        let vertex_offset = positions.len() as u32;

                        for vertex in FACE_POSITIONS[face] {
                            positions.push([
                                vertex[0] + x as f32,
                                vertex[1] + y as f32,
                                vertex[2] + z as f32,
                            ]);
                        }

                        normals.extend([FACE_NORMALS[face]; 4]);

                        uvs.extend([
                            [0.0, 1.0],
                            [1.0, 1.0],
                            [1.0, 0.0],
                            [0.0, 0.0]
                        ]);

                        for index in FACE_INDICES[face] {
                            indices.push(vertex_offset + index);
                        }
                    }
                }
            }
        }

        let mesh = Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices));

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_xyz((chunk_pos.0.x * CHUNK_SIZE as i32) as f32, 0.0, (chunk_pos.0.y * CHUNK_SIZE as i32) as f32),
        ));
    }
}

fn get_neighbor(manager: &ChunkManager, chunk_pos: &ChunkPos, chunk: &Chunk, x: i32, y: i32, z: i32) -> Blocks {
    if y < 0 || y >= (CHUNK_HEIGHT as i32) {
        return Blocks::Air;
    }

    let size = CHUNK_SIZE as i32;
    let world_x = chunk_pos.0.x * size + x;
    let world_z = chunk_pos.0.y * size + z;

    let chunk_x = world_x.div_euclid(size);
    let chunk_z = world_z.div_euclid(size);

    let local_x = world_x.rem_euclid(size) as usize;
    let local_z = world_z.rem_euclid(size) as usize;

    let target_chunk = if chunk_x == chunk_pos.0.x && chunk_z == chunk_pos.0.y {
        chunk
    } else {
        match manager.get_from_xz(chunk_x, chunk_z){
            Some(neighbor_chunk) => neighbor_chunk,
            None => return Blocks::Air,
        }
    };

    Blocks::from_u8(
        target_chunk
            .get(local_x, y as usize, local_z)
            .unwrap_or(Blocks::Air as u8)
    ).unwrap_or(Blocks::Air)
}

const FACE_POSITIONS: [[[f32; 3]; 4]; 6] = [
    // top (facing towards +y)
    [
        [-0.5, 0.5, -0.5], // vertex with index 0
        [0.5, 0.5, -0.5],  // vertex with index 1
        [0.5, 0.5, 0.5],   // etc. until 23
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

const FACE_NEIGHBORS: [[i32; 3]; 6] = [
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
