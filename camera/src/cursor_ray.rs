use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use world_api::{ActiveCamera, CursorRay};

pub struct CursorRayPlugin;

impl Plugin for CursorRayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CursorRay>()
            .add_systems(Update, emit_cursor_ray);
    }
}

fn emit_cursor_ray(
    camera_q: Query<(&Camera, &GlobalTransform), With<ActiveCamera>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut writer: MessageWriter<CursorRay>,
) {
    let Ok((camera, cam_transform)) = camera_q.single() else { return };
    let Ok(window) = window_q.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor_pos) else { return };

    writer.write(CursorRay {
        origin: ray.origin,
        direction: *ray.direction,
    });
}