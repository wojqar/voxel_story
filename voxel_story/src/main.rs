use bevy::prelude::*;
use bevy::window::PresentMode;
use camera::CameraPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CameraPlugin)
        .add_plugins(UiPlugin)
        .run();
}