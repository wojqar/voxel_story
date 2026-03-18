mod debug;
mod diagnostics;

pub use debug::DebugUiPlugin;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugUiPlugin)
            .add_plugins(diagnostics::DiagnosticsPlugin);
    }
}
