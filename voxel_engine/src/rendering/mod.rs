use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    world::VoxelWorld,
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, remesh_dirty_chunks);
    }
}

#[derive(Component)]
pub struct NeedsRemesh;

#[derive(Component)]
pub struct ChunkEntity(pub IVec3);

pub struct ChunkNeighbors<'a> {
    pub px: Option<&'a Chunk>,
    pub nx: Option<&'a Chunk>,
    pub py: Option<&'a Chunk>,
    pub ny: Option<&'a Chunk>,
    pub pz: Option<&'a Chunk>,
    pub nz: Option<&'a Chunk>,
}

fn is_transparent(chunk: &Chunk, neighbors: &ChunkNeighbors, x: i32, y: i32, z: i32) -> bool {
    let s = CHUNK_SIZE as i32;

    let in_bounds = x >= 0 && x < s && y >= 0 && y < s && z >= 0 && z < s;

    if in_bounds {
        return chunk.get(x as usize, y as usize, z as usize).is_air();
    }

    let neighbor = if x < 0 {
        neighbors.nx
    } else if x >= s {
        neighbors.px
    } else if y < 0 {
        neighbors.ny
    } else if y >= s {
        neighbors.py
    } else if z < 0 {
        neighbors.nz
    } else {
        neighbors.pz
    };

    match neighbor {
        None => true, // brak chunka = zakładamy solidny
        Some(n) => {
            let lx = x.rem_euclid(s) as usize;
            let ly = y.rem_euclid(s) as usize;
            let lz = z.rem_euclid(s) as usize;
            n.get(lx, ly, lz).is_air()
        }
    }
}

pub fn build_chunk_mesh(chunk: &Chunk, neighbors: &ChunkNeighbors) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get(x, y, z).is_air() {
                    continue;
                }

                let (ix, iy, iz) = (x as i32, y as i32, z as i32);
                let (fx, fy, fz) = (x as f32, y as f32, z as f32);

                // +X
                if is_transparent(chunk, neighbors, ix + 1, iy, iz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx + 1., fy, fz],
                        [fx + 1., fy, fz + 1.],
                        [fx + 1., fy + 1., fz + 1.],
                        [fx + 1., fy + 1., fz],
                        [1., 0., 0.],
                    );
                }
                // -X
                if is_transparent(chunk, neighbors, ix - 1, iy, iz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx, fy, fz + 1.],
                        [fx, fy, fz],
                        [fx, fy + 1., fz],
                        [fx, fy + 1., fz + 1.],
                        [-1., 0., 0.],
                    );
                }
                // +Y
                if is_transparent(chunk, neighbors, ix, iy + 1, iz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx, fy + 1., fz],
                        [fx + 1., fy + 1., fz],
                        [fx + 1., fy + 1., fz + 1.],
                        [fx, fy + 1., fz + 1.],
                        [0., 1., 0.],
                    );
                }
                // -Y
                if is_transparent(chunk, neighbors, ix, iy - 1, iz) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx, fy, fz + 1.],
                        [fx, fy, fz],
                        [fx + 1., fy, fz],
                        [fx + 1., fy, fz + 1.],
                        [0., -1., 0.],
                    );
                }
                // +Z
                if is_transparent(chunk, neighbors, ix, iy, iz + 1) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx + 1., fy, fz + 1.],
                        [fx, fy, fz + 1.],
                        [fx, fy + 1., fz + 1.],
                        [fx + 1., fy + 1., fz + 1.],
                        [0., 0., 1.],
                    );
                }
                // -Z
                if is_transparent(chunk, neighbors, ix, iy, iz - 1) {
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [fx, fy, fz],
                        [fx + 1., fy, fz],
                        [fx + 1., fy + 1., fz],
                        [fx, fy + 1., fz],
                        [0., 0., -1.],
                    );
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    normal: [f32; 3],
) {
    let base = positions.len() as u32;
    positions.extend_from_slice(&[v0, v1, v2, v3]);
    normals.extend_from_slice(&[normal; 4]);
    uvs.extend_from_slice(&[[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
    indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
}

fn remesh_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &ChunkEntity), With<NeedsRemesh>>,
    world: Res<VoxelWorld>,
) {
    for (entity, chunk_entity) in query.iter() {
        let coord = chunk_entity.0;
        let Some(chunk) = world.get_chunk(coord) else {
            continue;
        };

        let neighbors = ChunkNeighbors {
            px: world.get_chunk(coord + IVec3::X),
            nx: world.get_chunk(coord - IVec3::X),
            py: world.get_chunk(coord + IVec3::Y),
            ny: world.get_chunk(coord - IVec3::Y),
            pz: world.get_chunk(coord + IVec3::Z),
            nz: world.get_chunk(coord - IVec3::Z),
        };

        let mesh = meshes.add(build_chunk_mesh(chunk, &neighbors));
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.42, 0.72, 0.28),
            perceptual_roughness: 0.9,
            ..default()
        });
        let transform = Transform::from_translation((coord * CHUNK_SIZE as i32).as_vec3());

        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material), transform))
            .remove::<NeedsRemesh>();
    }
}
