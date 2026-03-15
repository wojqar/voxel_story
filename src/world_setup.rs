use bevy::light::light_consts::lux;
use bevy::pbr::{Atmosphere, ScatteringMedium};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use rts_camera::RtsCamera;
use spectator::{SpectatorActive, SpectatorCamera};
use voxel_engine::{
    VoxelWorld, WorldConfig,
    generation::{WorldGenerator, island::IslandGenerator},
    rendering::{ChunkEntity, NeedsRemesh},
};

pub struct WorldSetupPlugin;

impl Plugin for WorldSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut world: ResMut<VoxelWorld>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    let cfg = WorldConfig::new();
    let generator = IslandGenerator::from_config(&cfg);

    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(cfg.camera_pos()).looking_at(cfg.camera_target(), Vec3::Y),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
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
        RtsCamera {
            pivot: cfg.camera_target(),
            ..default()
        },
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.)),
    ));

    for cx in 0..cfg.chunks.x {
        for cz in 0..cfg.chunks.z {
            for cy in 0..cfg.chunks.y {
                let coord = IVec3::new(cx, cy, cz);
                world.insert_chunk(coord, generator.generate_chunk(coord));
                commands.spawn((ChunkEntity(coord), NeedsRemesh));
            }
        }
    }
}
