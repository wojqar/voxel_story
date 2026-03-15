use bevy::prelude::*;
use voxel_engine::{
    RenderingPlugin, VoxelEnginePlugin, VoxelWorld,
    generation::{WorldGenerator, heightmap::HeightmapGenerator},
    rendering::{ChunkEntity, NeedsRemesh},
};
use debug_ui::DebugUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((VoxelEnginePlugin, RenderingPlugin, DebugUiPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, mut world: ResMut<VoxelWorld>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-120., 100., -120.).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.4, 0.)),
    ));

    let generator = HeightmapGenerator::new(42);

    for cx in 0..8i32 {
        for cz in 0..8i32 {
            for cy in 0..3i32 {
                let coord = IVec3::new(cx, cy, cz);
                let chunk = generator.generate_chunk(coord);
                world.insert_chunk(coord, chunk);
                commands.spawn((ChunkEntity(coord), NeedsRemesh));
            }
        }
    }
}