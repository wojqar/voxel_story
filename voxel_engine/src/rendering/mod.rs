use crate::{
    chunk::{CHUNK_SIZE, Chunk},
    world::VoxelWorld,
};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chunk_material)
            .add_systems(Update, spawn_mesh_tasks)
            .add_systems(Update, apply_mesh_tasks.after(spawn_mesh_tasks));
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

#[derive(Component)]
pub struct MeshTask(Task<Mesh>);

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

// (dim, u_dim, v_dim, sign, winding)
// dim   = oś normalnej
// u_dim = oś szerokości w plasterku
// v_dim = oś wysokości w plasterku
// sign  = kierunek normalnej (+1 lub -1)
// winding = kolejność wierzchołków CCW widziana od zewnątrz
const FACES: [(usize, usize, usize, i32, [usize; 4]); 6] = [
    (0, 2, 1, 1, [1, 0, 3, 2]),  // +X
    (0, 2, 1, -1, [0, 1, 2, 3]), // -X
    (1, 0, 2, 1, [0, 3, 2, 1]),  // +Y
    (1, 0, 2, -1, [3, 0, 1, 2]), // -Y
    (2, 0, 1, 1, [0, 1, 2, 3]),  // +Z
    (2, 0, 1, -1, [1, 0, 3, 2]), // -Z
];
const MAX_MESH_TASKS_PER_FRAME: usize = 16;

pub fn build_chunk_mesh(chunk: &Chunk, neighbors: &ChunkNeighbors) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for (dim, u_dim, v_dim, sign, winding) in FACES {
        let normal = {
            let mut n = [0.0f32; 3];
            n[dim] = sign as f32;
            n
        };

        for d in 0..CHUNK_SIZE {
            let mut mask = [[crate::voxel::VoxelId::AIR; CHUNK_SIZE]; CHUNK_SIZE];
            let mut visited = [[false; CHUNK_SIZE]; CHUNK_SIZE];

            // Buduj maskę widocznych ścian w tym plasterku
            for u in 0..CHUNK_SIZE {
                for v in 0..CHUNK_SIZE {
                    let mut pos = [0i32; 3];
                    pos[dim] = d as i32;
                    pos[u_dim] = u as i32;
                    pos[v_dim] = v as i32;

                    let voxel = chunk.get(pos[0] as usize, pos[1] as usize, pos[2] as usize);
                    if voxel.is_air() {
                        continue;
                    }

                    let mut nb = pos;
                    nb[dim] += sign;

                    if is_transparent(chunk, neighbors, nb[0], nb[1], nb[2]) {
                        mask[u][v] = voxel;
                    }
                }
            }

            // Greedy — znajdź największe prostokąty
            for u in 0..CHUNK_SIZE {
                for v in 0..CHUNK_SIZE {
                    if visited[u][v] {
                        continue;
                    }
                    let voxel = mask[u][v];
                    if voxel.is_air() {
                        continue;
                    }

                    // Rozszerz w v
                    let mut width = 1;
                    while v + width < CHUNK_SIZE
                        && !visited[u][v + width]
                        && mask[u][v + width] == voxel
                    {
                        width += 1;
                    }

                    // Rozszerz w u
                    let mut height = 1;
                    'outer: while u + height < CHUNK_SIZE {
                        for k in 0..width {
                            if visited[u + height][v + k] || mask[u + height][v + k] != voxel {
                                break 'outer;
                            }
                        }
                        height += 1;
                    }

                    // Oznacz odwiedzone
                    for hu in 0..height {
                        for hv in 0..width {
                            visited[u + hu][v + hv] = true;
                        }
                    }

                    // Oblicz wierzchołki
                    let fd = d as f32 + if sign == 1 { 1.0 } else { 0.0 };
                    let fu = u as f32;
                    let fv = v as f32;
                    let fh = height as f32;
                    let fw = width as f32;

                    let corners = [(fu, fv), (fu + fh, fv), (fu + fh, fv + fw), (fu, fv + fw)];
                    let mut p = [[0.0f32; 3]; 4];
                    for (i, (cu, cv)) in corners.iter().enumerate() {
                        p[i][dim] = fd;
                        p[i][u_dim] = *cu;
                        p[i][v_dim] = *cv;
                    }

                    let verts = [p[0], p[1], p[2], p[3]];
                    let [v0, v1, v2, v3] = [
                        verts[winding[0]],
                        verts[winding[1]],
                        verts[winding[2]],
                        verts[winding[3]],
                    ];

                    let color = voxel.color();
                    let base = positions.len() as u32;

                    positions.extend_from_slice(&[v0, v1, v2, v3]);
                    normals.extend_from_slice(&[normal; 4]);
                    uvs.extend_from_slice(&[[0.0, 0.0], [fh, 0.0], [fh, fw], [0.0, fw]]);
                    colors.extend_from_slice(&[color; 4]);
                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
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

fn spawn_mesh_tasks(
    mut commands: Commands,
    query: Query<(Entity, &ChunkEntity), With<NeedsRemesh>>,
    world: Res<VoxelWorld>,
) {
    let pool = AsyncComputeTaskPool::get();
    let mut budget = MAX_MESH_TASKS_PER_FRAME;

    for (entity, chunk_entity) in query.iter() {
        if budget == 0 {
            break;
        }

        let coord = chunk_entity.0;
        let Some(chunk) = world.get_chunk(coord) else {
            continue;
        };

        if chunk.count_solid() == 0 {
            commands
                .entity(entity)
                .remove::<NeedsRemesh>()
                .remove::<MeshTask>()
                .remove::<Mesh3d>()
                .remove::<MeshMaterial3d<StandardMaterial>>();
            continue;
        }

        let chunk = chunk.clone();
        let px = world.get_chunk(coord + IVec3::X).cloned();
        let nx = world.get_chunk(coord - IVec3::X).cloned();
        let py = world.get_chunk(coord + IVec3::Y).cloned();
        let ny = world.get_chunk(coord - IVec3::Y).cloned();
        let pz = world.get_chunk(coord + IVec3::Z).cloned();
        let nz = world.get_chunk(coord - IVec3::Z).cloned();

        let task = pool.spawn(async move {
            let neighbors = ChunkNeighbors {
                px: px.as_ref(),
                nx: nx.as_ref(),
                py: py.as_ref(),
                ny: ny.as_ref(),
                pz: pz.as_ref(),
                nz: nz.as_ref(),
            };
            build_chunk_mesh(&chunk, &neighbors)
        });

        commands
            .entity(entity)
            .insert(MeshTask(task))
            .remove::<NeedsRemesh>();

        budget -= 1;
    }
}

fn apply_mesh_tasks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterial>,
    mut query: Query<(Entity, &ChunkEntity, &mut MeshTask)>,
) {
    for (entity, chunk_entity, mut task) in query.iter_mut() {
        let Some(mesh) = future::block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        let coord = chunk_entity.0;
        let mesh = meshes.add(mesh);
        let transform = Transform::from_translation((coord * CHUNK_SIZE as i32).as_vec3());

        commands
            .entity(entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material.0.clone()), transform))
            .remove::<MeshTask>();
    }
}
