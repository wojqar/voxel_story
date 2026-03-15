use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use debug_ui::DebugMetrics;
use rts_camera::RtsActive;
use voxel_engine::{
    VoxelId, VoxelWorld,
    chunk::CHUNK_SIZE,
    rendering::{ChunkEntity, NeedsRemesh},
};

const MAX_REACH: f32 = 60.0;

#[derive(Resource, Default)]
pub struct BlockTarget {
    pub block: Option<IVec3>,
    pub normal: IVec3,
}

#[derive(Resource)]
pub struct SelectedVoxel(pub VoxelId);

impl Default for SelectedVoxel {
    fn default() -> Self {
        Self(VoxelId::STONE)
    }
}

#[derive(Component)]
struct BlockHighlight;

pub struct EditingPlugin;

impl Plugin for EditingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockTarget>()
            .init_resource::<SelectedVoxel>()
            .add_systems(Startup, spawn_highlight)
            .add_systems(
                Update,
                (
                    raycast_target,
                    update_highlight.after(raycast_target),
                    handle_editing
                        .after(raycast_target)
                        .run_if(|q: Query<(), With<RtsActive>>| !q.is_empty()),
                    select_voxel_type,
                    update_editing_metrics,
                ),
            );
    }
}

fn spawn_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.012, 1.012, 1.012));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.9, 0.2, 0.3),
        emissive: LinearRgba::new(0.6, 0.5, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        BlockHighlight,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Hidden,
    ));
}

fn raycast_target(
    rts_active: Query<(), With<RtsActive>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<RtsActive>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    world: Res<VoxelWorld>,
    mut target: ResMut<BlockTarget>,
) {
    if rts_active.is_empty() {
        target.block = None;
        return;
    }

    let (Ok((camera, cam_transform)), Ok(window)) = (camera_q.single(), windows.single()) else {
        target.block = None;
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        target.block = None;
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else {
        target.block = None;
        return;
    };

    match voxel_raycast(&world, ray.origin, ray.direction.into(), MAX_REACH) {
        Some((block, normal)) => {
            target.block = Some(block);
            target.normal = normal;
        }
        None => target.block = None,
    }
}

fn update_highlight(
    target: Res<BlockTarget>,
    mut highlight_q: Query<(&mut Transform, &mut Visibility), With<BlockHighlight>>,
) {
    let Ok((mut transform, mut visibility)) = highlight_q.single_mut() else {
        return;
    };

    match target.block {
        Some(pos) => {
            *visibility = Visibility::Visible;
            transform.translation = pos.as_vec3() + Vec3::splat(0.5);
        }
        None => *visibility = Visibility::Hidden,
    }
}

fn handle_editing(
    mouse: Res<ButtonInput<MouseButton>>,
    target: Res<BlockTarget>,
    selected: Res<SelectedVoxel>,
    mut world: ResMut<VoxelWorld>,
    chunk_q: Query<(Entity, &ChunkEntity)>,
    mut commands: Commands,
) {
    let Some(pos) = target.block else { return };

    if mouse.just_pressed(MouseButton::Left) {
        if world.set_voxel(pos, VoxelId::AIR) {
            mark_remesh(pos, &chunk_q, &mut commands);
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        let place_pos = pos + target.normal;
        if world.set_voxel(place_pos, selected.0) {
            mark_remesh(place_pos, &chunk_q, &mut commands);
        }
    }
}

fn select_voxel_type(keys: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedVoxel>) {
    if keys.just_pressed(KeyCode::Digit1) {
        selected.0 = VoxelId::STONE;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        selected.0 = VoxelId::DIRT;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        selected.0 = VoxelId::GRASS;
    }
}

fn update_editing_metrics(
    selected: Res<SelectedVoxel>,
    target: Res<BlockTarget>,
    mut metrics: ResMut<DebugMetrics>,
) {
    let name = match selected.0 {
        VoxelId::STONE => "Stone",
        VoxelId::DIRT => "Dirt",
        VoxelId::GRASS => "Grass",
        _ => "Unknown",
    };
    metrics.set("Edit", "Selected", name);
    metrics.set(
        "Edit",
        "Target",
        match target.block {
            Some(p) => format!("{},{},{}", p.x, p.y, p.z),
            None => "-".to_string(),
        },
    );
}

fn mark_remesh(world_pos: IVec3, chunk_q: &Query<(Entity, &ChunkEntity)>, commands: &mut Commands) {
    let cs = CHUNK_SIZE as i32;
    let chunk_coord = IVec3::new(
        world_pos.x.div_euclid(cs),
        world_pos.y.div_euclid(cs),
        world_pos.z.div_euclid(cs),
    );

    let lx = world_pos.x.rem_euclid(cs);
    let ly = world_pos.y.rem_euclid(cs);
    let lz = world_pos.z.rem_euclid(cs);

    let mut dirty = vec![chunk_coord];
    if lx == 0 {
        dirty.push(chunk_coord - IVec3::X);
    }
    if lx == cs - 1 {
        dirty.push(chunk_coord + IVec3::X);
    }
    if ly == 0 {
        dirty.push(chunk_coord - IVec3::Y);
    }
    if ly == cs - 1 {
        dirty.push(chunk_coord + IVec3::Y);
    }
    if lz == 0 {
        dirty.push(chunk_coord - IVec3::Z);
    }
    if lz == cs - 1 {
        dirty.push(chunk_coord + IVec3::Z);
    }

    for (entity, ce) in chunk_q.iter() {
        if dirty.contains(&ce.0) {
            commands.entity(entity).insert(NeedsRemesh);
        }
    }
}

fn voxel_raycast(
    world: &VoxelWorld,
    origin: Vec3,
    dir: Vec3,
    max_dist: f32,
) -> Option<(IVec3, IVec3)> {
    let dir = dir.normalize();

    let mut pos = IVec3::new(
        origin.x.floor() as i32,
        origin.y.floor() as i32,
        origin.z.floor() as i32,
    );

    let step = IVec3::new(
        if dir.x >= 0.0 { 1 } else { -1 },
        if dir.y >= 0.0 { 1 } else { -1 },
        if dir.z >= 0.0 { 1 } else { -1 },
    );

    let t_delta = Vec3::new(
        if dir.x != 0.0 {
            1.0 / dir.x.abs()
        } else {
            f32::MAX
        },
        if dir.y != 0.0 {
            1.0 / dir.y.abs()
        } else {
            f32::MAX
        },
        if dir.z != 0.0 {
            1.0 / dir.z.abs()
        } else {
            f32::MAX
        },
    );

    let mut t_max = Vec3::new(
        if dir.x > 0.0 {
            ((pos.x + 1) as f32 - origin.x) / dir.x
        } else if dir.x < 0.0 {
            (pos.x as f32 - origin.x) / dir.x
        } else {
            f32::MAX
        },
        if dir.y > 0.0 {
            ((pos.y + 1) as f32 - origin.y) / dir.y
        } else if dir.y < 0.0 {
            (pos.y as f32 - origin.y) / dir.y
        } else {
            f32::MAX
        },
        if dir.z > 0.0 {
            ((pos.z + 1) as f32 - origin.z) / dir.z
        } else if dir.z < 0.0 {
            (pos.z as f32 - origin.z) / dir.z
        } else {
            f32::MAX
        },
    );

    let mut normal = IVec3::ZERO;

    if is_solid(world, pos) {
        return Some((pos, normal));
    }

    loop {
        if t_max.x < t_max.y && t_max.x < t_max.z {
            if t_max.x > max_dist {
                break;
            }
            pos.x += step.x;
            normal = IVec3::new(-step.x, 0, 0);
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.z {
            if t_max.y > max_dist {
                break;
            }
            pos.y += step.y;
            normal = IVec3::new(0, -step.y, 0);
            t_max.y += t_delta.y;
        } else {
            if t_max.z > max_dist {
                break;
            }
            pos.z += step.z;
            normal = IVec3::new(0, 0, -step.z);
            t_max.z += t_delta.z;
        }

        if is_solid(world, pos) {
            return Some((pos, normal));
        }
    }

    None
}

fn is_solid(world: &VoxelWorld, pos: IVec3) -> bool {
    let cs = CHUNK_SIZE as i32;
    let chunk_coord = IVec3::new(
        pos.x.div_euclid(cs),
        pos.y.div_euclid(cs),
        pos.z.div_euclid(cs),
    );
    let lx = pos.x.rem_euclid(cs) as usize;
    let ly = pos.y.rem_euclid(cs) as usize;
    let lz = pos.z.rem_euclid(cs) as usize;

    world
        .get_chunk(chunk_coord)
        .map_or(false, |c| !c.get(lx, ly, lz).is_air())
}
