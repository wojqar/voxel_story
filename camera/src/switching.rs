use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use world_api::ActiveCamera;

use crate::rts::{RtsActive, RtsCamera};
use crate::spectator::{SpectatorActive, SpectatorCamera};

pub struct SwitchingPlugin;

impl Plugin for SwitchingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraMode>()
            .add_systems(Update, switch);
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CameraMode {
    #[default]
    Spectator,
    Rts,
}

fn switch(
    keys: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<CameraMode>,
    mut commands: Commands,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut spectator_q: Query<(Entity, &mut SpectatorCamera, &Transform)>,
    rts_q: Query<Entity, With<RtsCamera>>,
) {
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }

    match *mode {
        CameraMode::Spectator => {
            for (e, _, _) in spectator_q.iter() {
                commands
                    .entity(e)
                    .remove::<SpectatorActive>()
                    .remove::<ActiveCamera>();
            }
            for e in rts_q.iter() {
                commands.entity(e).insert(RtsActive).insert(ActiveCamera);
            }
            cursor.grab_mode = CursorGrabMode::None;
            cursor.visible = true;
            *mode = CameraMode::Rts;
        }
        CameraMode::Rts => {
            for e in rts_q.iter() {
                commands
                    .entity(e)
                    .remove::<RtsActive>()
                    .remove::<ActiveCamera>();
            }
            for (e, mut spec, transform) in spectator_q.iter_mut() {
                let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                spec.yaw = yaw.to_degrees();
                spec.pitch = pitch.to_degrees();
                commands
                    .entity(e)
                    .insert(SpectatorActive)
                    .insert(ActiveCamera);
            }
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
            *mode = CameraMode::Spectator;
        }
    }
}
