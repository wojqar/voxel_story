mod collect;
mod panel;

use bevy::prelude::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};
use world_api::{DebugEntry, DebugMetrics};

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_message::<DebugEntry>()
            .init_resource::<DebugMetrics>()
            .add_systems(Update, collect::collect_debug_entries)
            .add_systems(EguiPrimaryContextPass, panel::draw_debug_panel);
    }
}