use bevy::app::PostStartup;
use bevy::light::light_consts::lux;
use bevy::prelude::*;
use world_api::{ActiveCamera, MainTerrainAnchor};

use crate::rts::{RtsCamera, apply_rts_transform};
use crate::spectator::{SpectatorActive, SpectatorCamera};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.5, 0.67, 0.85)))
            .add_systems(Startup, spawn_camera)
            .add_systems(PostStartup, align_camera_to_main_terrain)
            .add_systems(Startup, spawn_sun);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 80.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        SpectatorCamera::default(),
        SpectatorActive,
        RtsCamera::default(),
        ActiveCamera,
    ));
}

fn align_camera_to_main_terrain(
    anchor: Res<MainTerrainAnchor>,
    mut camera_q: Query<(&mut Transform, &mut RtsCamera, &mut SpectatorCamera), With<Camera3d>>,
) {
    let Ok((mut transform, mut rts, mut spectator)) = camera_q.single_mut() else {
        return;
    };

    rts.pivot = anchor.focus;
    rts.pivot_y_target = anchor.focus.y;
    apply_rts_transform(&rts, &mut transform);

    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    spectator.yaw = yaw.to_degrees();
    spectator.pitch = pitch.to_degrees();
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
    ));
}
