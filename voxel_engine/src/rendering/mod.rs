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
        app.add_systems(Startup, setup_chunk_material)
            .add_systems(Update, remesh_dirty_chunks);
    }
}

#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<StandardMaterial>);

fn setup_chunk_material(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let handle = materials.add(StandardMaterial {
        perceptual_roughness: 0.9,
        ..default()
    });
    commands.insert_resource(ChunkMaterial(handle));
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
    if x >= 0 && x < s && y >= 0 && y < s && z >= 0 && z < s {
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
        None => true,
        Some(n) => n
            .get(
                x.rem_euclid(s) as usize,
                y.rem_euclid(s) as usize,
                z.rem_euclid(s) as usize,
            )
            .is_air(),
    }
}

pub fn build_chunk_mesh(chunk: &Chunk, neighbors: &ChunkNeighbors) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);
                if voxel.is_air() {
                    continue;
                }

                let (ix, iy, iz) = (x as i32, y as i32, z as i32);
                let (fx, fy, fz) = (x as f32, y as f32, z as f32);

                let faces: [([f32; 3], [f32; 3], [f32; 3], [f32; 3], [f32; 3]); 6] = [
                    // +X (widok od +X, CCW)
                    (
                        [fx + 1., fy, fz + 1.],
                        [fx + 1., fy, fz],
                        [fx + 1., fy + 1., fz],
                        [fx + 1., fy + 1., fz + 1.],
                        [1., 0., 0.],
                    ),
                    // -X (widok od -X, CCW)
                    (
                        [fx, fy, fz],
                        [fx, fy, fz + 1.],
                        [fx, fy + 1., fz + 1.],
                        [fx, fy + 1., fz],
                        [-1., 0., 0.],
                    ),
                    // +Y (widok od +Y, CCW)
                    (
                        [fx, fy + 1., fz],
                        [fx, fy + 1., fz + 1.],
                        [fx + 1., fy + 1., fz + 1.],
                        [fx + 1., fy + 1., fz],
                        [0., 1., 0.],
                    ),
                    // -Y (widok od -Y, CCW) — FIX
                    (
                        [fx, fy, fz],
                        [fx + 1., fy, fz],
                        [fx + 1., fy, fz + 1.],
                        [fx, fy, fz + 1.],
                        [0., -1., 0.],
                    ),
                    // +Z (widok od +Z, CCW)
                    (
                        [fx, fy, fz + 1.],
                        [fx + 1., fy, fz + 1.],
                        [fx + 1., fy + 1., fz + 1.],
                        [fx, fy + 1., fz + 1.],
                        [0., 0., 1.],
                    ),
                    // -Z (widok od -Z, CCW)
                    (
                        [fx + 1., fy, fz],
                        [fx, fy, fz],
                        [fx, fy + 1., fz],
                        [fx + 1., fy + 1., fz],
                        [0., 0., -1.],
                    ),
                ];

                let offsets: [(i32, i32, i32); 6] = [
                    (ix + 1, iy, iz),
                    (ix - 1, iy, iz),
                    (ix, iy + 1, iz),
                    (ix, iy - 1, iz),
                    (ix, iy, iz + 1),
                    (ix, iy, iz - 1),
                ];

                for (i, (v0, v1, v2, v3, normal)) in faces.iter().enumerate() {
                    let (ox, oy, oz) = offsets[i];
                    if !is_transparent(chunk, neighbors, ox, oy, oz) {
                        continue;
                    }
                    let color = voxel.color();
                    add_quad(
                        &mut positions,
                        &mut normals,
                        &mut uvs,
                        &mut colors,
                        &mut indices,
                        *v0,
                        *v1,
                        *v2,
                        *v3,
                        *normal,
                        color,
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn add_quad(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
) {
    let base = positions.len() as u32;
    positions.extend_from_slice(&[v0, v1, v2, v3]);
    normals.extend_from_slice(&[normal; 4]);
    uvs.extend_from_slice(&[[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
    colors.extend_from_slice(&[color; 4]);
    // CCW: (v0,v1,v2), (v0,v2,v3)
    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn remesh_dirty_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterial>,
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
        let transform = Transform::from_translation((coord * CHUNK_SIZE as i32).as_vec3());

        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material.0.clone()), transform))
            .remove::<NeedsRemesh>();
    }
}
