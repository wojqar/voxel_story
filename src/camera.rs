use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use rts_camera::{RtsActive, RtsCamera};
use spectator::{SpectatorActive, SpectatorCamera};
use voxel_engine::{VoxelWorld, chunk::CHUNK_SIZE};

const RING_INNER_RATIO: f32 = 0.75;
const RING_OUTER_RADIUS: f32 = 0.8;
const RING_Y_OFFSET: f32 = 0.08;
const WORLD_HEIGHT: i32 = 128;

#[derive(Component)]
struct PivotMarker;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (grab_cursor, spawn_pivot_marker))
            .add_systems(Update, camera_switch)
            .add_systems(
                Update,
                (rts_pivot_y, update_pivot_marker)
                    .chain()
                    .run_if(|q: Query<(), With<RtsActive>>| !q.is_empty()),
            );
    }
}

fn grab_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn spawn_pivot_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let inner = RING_OUTER_RADIUS * RING_INNER_RATIO;
    let mesh = meshes.add(
        Annulus::new(inner, RING_OUTER_RADIUS)
            .mesh()
            .resolution(64)
            .build(),
    );
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.85, 0.1, 0.9),
        emissive: LinearRgba::new(0.8, 0.6, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    commands.spawn((
        PivotMarker,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::default(),
        Visibility::Hidden,
    ));
}

fn camera_switch(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut spectator_q: Query<(Entity, &mut SpectatorCamera, &Transform), With<SpectatorCamera>>,
    rts_q: Query<Entity, With<RtsCamera>>,
    active_spectator: Query<(), With<SpectatorActive>>,
    mut marker_q: Query<&mut Visibility, With<PivotMarker>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    let is_spectator = !active_spectator.is_empty();

    if is_spectator {
        for (e, _, _) in spectator_q.iter() {
            commands.entity(e).remove::<SpectatorActive>();
        }
        for e in rts_q.iter() {
            commands.entity(e).insert(RtsActive);
        }
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
        for mut vis in marker_q.iter_mut() {
            *vis = Visibility::Visible;
        }
    } else {
        for e in rts_q.iter() {
            commands.entity(e).remove::<RtsActive>();
        }
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
        for mut vis in marker_q.iter_mut() {
            *vis = Visibility::Hidden;
        }
        for (e, mut spec, transform) in spectator_q.iter_mut() {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            spec.yaw = yaw.to_degrees();
            spec.pitch = pitch.to_degrees();
            commands.entity(e).insert(SpectatorActive);
        }
    }
}

fn rts_pivot_y(
    world: Res<VoxelWorld>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    for mut cam in query.iter_mut() {
        if keys.pressed(KeyCode::PageUp) {
            cam.pivot.y += cam.manual_y_speed * time.delta_secs();
            cam.raycast_active = false;
        } else if keys.pressed(KeyCode::PageDown) {
            cam.pivot.y -= cam.manual_y_speed * time.delta_secs();
            cam.raycast_active = false;
        }

        if !cam.raycast_active {
            continue;
        }

        if let Some(y) = ground_at(&world, cam.pivot.x, cam.pivot.z) {
            cam.pivot_y_target = y;
        }

        let t = (cam.pivot_y_lerp_speed * time.delta_secs()).min(1.0);
        cam.pivot.y += (cam.pivot_y_target - cam.pivot.y) * t;
    }
}

fn update_pivot_marker(
    cam_q: Query<&RtsCamera, With<RtsActive>>,
    mut marker_q: Query<&mut Transform, With<PivotMarker>>,
) {
    let Ok(cam) = cam_q.single() else { return };
    let Ok(mut transform) = marker_q.single_mut() else {
        return;
    };

    transform.translation = cam.pivot + Vec3::Y * RING_Y_OFFSET;
    transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    transform.scale = Vec3::ONE;
}

fn ground_at(world: &VoxelWorld, wx: f32, wz: f32) -> Option<f32> {
    let x = wx as i32;
    let z = wz as i32;
    let cs = CHUNK_SIZE as i32;

    for wy in (0..WORLD_HEIGHT).rev() {
        let chunk_coord = IVec3::new(x.div_euclid(cs), wy.div_euclid(cs), z.div_euclid(cs));
        let Some(chunk) = world.get_chunk(chunk_coord) else {
            continue;
        };

        let lx = x.rem_euclid(cs) as usize;
        let ly = wy.rem_euclid(cs) as usize;
        let lz = z.rem_euclid(cs) as usize;

        if !chunk.get(lx, ly, lz).is_air() {
            return Some(wy as f32 + 1.0);
        }
    }
    None
}
