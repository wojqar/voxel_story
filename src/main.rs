mod camera;
mod diagnostics;
mod world_setup;

use bevy::prelude::*;
use camera::CameraPlugin;
use debug_ui::DebugUiPlugin;
use diagnostics::DiagnosticsPlugin;
use rts_camera::RtsCameraPlugin;
use spectator::SpectatorPlugin;
use voxel_engine::{RenderingPlugin, VoxelEnginePlugin};
use world_setup::WorldSetupPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            VoxelEnginePlugin,
            RenderingPlugin,
            DebugUiPlugin,
            SpectatorPlugin,
            RtsCameraPlugin,
        ))
        .add_plugins((
            WorldSetupPlugin,
            CameraPlugin,
            DiagnosticsPlugin,
        ))
        .run();
}