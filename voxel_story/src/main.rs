use bevy::prelude::*;
use camera::CameraPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(UiPlugin)
        .run();
}