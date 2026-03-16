mod cursor_ray;
mod rts;
mod setup;
mod spectator;
mod switching;

pub use rts::{RtsActive, RtsCamera};
pub use spectator::{SpectatorActive, SpectatorCamera};
pub use switching::CameraMode;

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            setup::SetupPlugin,
            spectator::SpectatorPlugin,
            rts::RtsCameraPlugin,
            switching::SwitchingPlugin,
            cursor_ray::CursorRayPlugin,
        ));
    }
}