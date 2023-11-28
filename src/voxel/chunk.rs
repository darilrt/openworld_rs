// use super::world::World as voxelWorld;
use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_rapier3d::geometry::Collider;

pub const CHUNK_SIZE: usize = 64;

#[derive(Component)]
pub struct Chunk {
    pub updated: bool,
    pub position: IVec3,
    pub blocks: Vec<Vec<Vec<u8>>>,
}

impl Chunk {
    pub fn new(position: IVec3) -> Self {
        Self {
            position,
            blocks: vec![vec![vec![0; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
            updated: false,
        }
    }
}

pub fn update(
    mut query: Query<(&mut Chunk, &mut Collider, &mut Handle<Mesh>), With<Chunk>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut world: ResMut<voxelWorld>,
) {
    for (mut chunk, mut collider, mesh_ref) in &mut query {
        if chunk.updated {
            continue;
        }

        let (mesh, colliders) = build_chunk_mesh(&chunk);

        *meshes.get_mut(mesh_ref.id()).unwrap() = mesh;
        collider.raw = colliders.raw;

        chunk.updated = true;
    }
}

fn build_chunk_mesh(chunk: &Chunk) -> (Mesh, Collider) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut colliders: Vec<(Vec3, Quat, Collider)> = vec![];

    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut colors: Vec<[f32; 4]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
    let mut indices: Vec<u32> = vec![];

    for ix in 0..CHUNK_SIZE {
        for iy in 0..CHUNK_SIZE {
            for iz in 0..CHUNK_SIZE {
                let block = chunk.blocks[ix][iy][iz];

                if block == 0 {
                    continue;
                }

                // add collider if any side is exposed
                if (ix == 0 || ix == CHUNK_SIZE - 1 || chunk.blocks[ix - 1][iy][iz] == 0)
                    || (ix == 0 || ix == CHUNK_SIZE - 1 || chunk.blocks[ix + 1][iy][iz] == 0)
                    || (iy == 0 || iy == CHUNK_SIZE - 1 || chunk.blocks[ix][iy - 1][iz] == 0)
                    || (iy == 0 || iy == CHUNK_SIZE - 1 || chunk.blocks[ix][iy + 1][iz] == 0)
                    || (iz == 0 || iz == CHUNK_SIZE - 1 || chunk.blocks[ix][iy][iz - 1] == 0)
                    || (iz == 0 || iz == CHUNK_SIZE - 1 || chunk.blocks[ix][iy][iz + 1] == 0)
                {
                    colliders.push((
                        Vec3::new(ix as f32 + 0.5, iy as f32 + 0.5, iz as f32 + 0.5),
                        Quat::IDENTITY,
                        Collider::cuboid(0.5, 0.5, 0.5),
                    ));
                }

                let x = ix as f32;
                let y = iy as f32;
                let z = iz as f32;

                let mut add_face = |face: [[f32; 3]; 4], normal: [f32; 3], uv: [[f32; 2]; 4]| {
                    const COLOR_WATER: [f32; 4] = [0.106, 0.192, 0.549, 1.0];
                    const COLOR_GRASS: [f32; 4] = [0.102, 0.631, 0.259, 1.0];

                    if block == 1 {
                        colors.extend(&[COLOR_WATER, COLOR_WATER, COLOR_WATER, COLOR_WATER]);
                    } else if block == 2 {
                        colors.extend(&[COLOR_GRASS, COLOR_GRASS, COLOR_GRASS, COLOR_GRASS]);
                    }

                    let index = positions.len() as u32;

                    positions.extend(&[
                        [x + face[0][0], y + face[0][1], z + face[0][2]],
                        [x + face[1][0], y + face[1][1], z + face[1][2]],
                        [x + face[2][0], y + face[2][1], z + face[2][2]],
                        [x + face[3][0], y + face[3][1], z + face[3][2]],
                    ]);

                    normals.extend(&[normal, normal, normal, normal]);

                    uvs.extend(&[
                        [uv[0][0], uv[0][1]],
                        [uv[1][0], uv[1][1]],
                        [uv[2][0], uv[2][1]],
                        [uv[3][0], uv[3][1]],
                    ]);

                    indices.extend(&[index, index + 1, index + 2, index, index + 2, index + 3]);
                };

                // Front
                if iz == 0 || chunk.blocks[ix][iy][iz - 1] == 0 {
                    add_face(
                        [
                            [0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0],
                            [1.0, 1.0, 0.0],
                            [1.0, 0.0, 0.0],
                        ],
                        [0.0, 0.0, -1.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }

                // Back
                if iz == 0 || iz == CHUNK_SIZE - 1 || chunk.blocks[ix][iy][iz + 1] == 0 {
                    add_face(
                        [
                            [1.0, 0.0, 1.0],
                            [1.0, 1.0, 1.0],
                            [0.0, 1.0, 1.0],
                            [0.0, 0.0, 1.0],
                        ],
                        [0.0, 0.0, 1.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }

                // Left
                if ix == 0 || chunk.blocks[ix - 1][iy][iz] == 0 {
                    add_face(
                        [
                            [0.0, 0.0, 1.0],
                            [0.0, 1.0, 1.0],
                            [0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0],
                        ],
                        [-1.0, 0.0, 0.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }

                // Right
                if ix == 0 || ix == CHUNK_SIZE - 1 || chunk.blocks[ix + 1][iy][iz] == 0 {
                    add_face(
                        [
                            [1.0, 0.0, 0.0],
                            [1.0, 1.0, 0.0],
                            [1.0, 1.0, 1.0],
                            [1.0, 0.0, 1.0],
                        ],
                        [1.0, 0.0, 0.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }

                // Top
                if iy == 0 || iy == CHUNK_SIZE - 1 || chunk.blocks[ix][iy + 1][iz] == 0 {
                    add_face(
                        [
                            [0.0, 1.0, 0.0],
                            [0.0, 1.0, 1.0],
                            [1.0, 1.0, 1.0],
                            [1.0, 1.0, 0.0],
                        ],
                        [0.0, 1.0, 0.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }

                // Bottom
                if iy == 0 || chunk.blocks[ix][iy - 1][iz] == 0 {
                    add_face(
                        [
                            [0.0, 0.0, 1.0],
                            [0.0, 0.0, 0.0],
                            [1.0, 0.0, 0.0],
                            [1.0, 0.0, 1.0],
                        ],
                        [0.0, -1.0, 0.0],
                        [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
                    );
                }
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(indices)));

    (mesh, Collider::compound(colliders))
}
