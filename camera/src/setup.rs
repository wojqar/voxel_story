use bevy::light::light_consts::lux;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::pbr::{Atmosphere, DistanceFog, FogFalloff, ScatteringMedium};
use bevy::prelude::*;
use world_api::{ActiveCamera, ChunkObserver};

use crate::rts::RtsCamera;
use crate::spectator::{SpectatorActive, SpectatorCamera};

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
           .add_systems(Startup, spawn_sun);
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut scattering: ResMut<Assets<ScatteringMedium>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 80.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::earthlike(scattering.add(ScatteringMedium::default())),
        DistanceFog {
            color: Color::srgba(0.5, 0.67, 0.85, 1.0),
            directional_light_color: Color::srgba(1.0, 0.95, 0.75, 0.5),
            directional_light_exponent: 30.0,
            falloff: FogFalloff::from_visibility_colors(
                400.0,
                Color::srgb(0.5, 0.67, 0.85),
                Color::srgb(0.8, 0.9, 1.0),
            ),
        },
        SpectatorCamera::default(),
        SpectatorActive,
        RtsCamera::default(),
        ActiveCamera,
        ChunkObserver::default(),
    ));
}

fn spawn_sun(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            first_cascade_far_bound: 30.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));
}