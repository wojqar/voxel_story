use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use rts_camera::{RtsActive, RtsCamera};
use spectator::{SpectatorActive, SpectatorCamera};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, grab_cursor)
            .add_systems(Update, camera_switch);
    }
}

fn grab_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn camera_switch(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    spectator_q: Query<Entity, With<SpectatorCamera>>,
    rts_q: Query<Entity, With<RtsCamera>>,
    active_spectator: Query<(), With<SpectatorActive>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    let is_spectator = !active_spectator.is_empty();

    if is_spectator {
        for e in spectator_q.iter() {
            commands.entity(e).remove::<SpectatorActive>();
        }
        for e in rts_q.iter() {
            commands.entity(e).insert(RtsActive);
        }
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    } else {
        for e in rts_q.iter() {
            commands.entity(e).remove::<RtsActive>();
        }
        for e in spectator_q.iter() {
            commands.entity(e).insert(SpectatorActive);
        }
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }
}