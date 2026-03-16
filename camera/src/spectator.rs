use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

pub struct SpectatorPlugin;

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (mouse_look, movement.after(mouse_look), scroll_speed),
        );
    }
}

#[derive(Component)]
pub struct SpectatorCamera {
    pub speed: f32,
    pub sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for SpectatorCamera {
    fn default() -> Self {
        Self {
            speed: 20.0,
            sensitivity: 0.1,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

/// Marker — spectator aktywny. Dodawany/usuwany przez SwitchingPlugin.
#[derive(Component)]
pub struct SpectatorActive;

fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut SpectatorCamera, &mut Transform), With<SpectatorActive>>,
) {
    let delta = motion.delta;
    if delta == Vec2::ZERO { return; }
    for (mut cam, mut transform) in query.iter_mut() {
        cam.yaw -= delta.x * cam.sensitivity;
        cam.pitch = (cam.pitch - delta.y * cam.sensitivity).clamp(-89.0, 89.0);
        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            cam.yaw.to_radians(),
            cam.pitch.to_radians(),
            0.0,
        );
    }
}

fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&SpectatorCamera, &mut Transform), With<SpectatorActive>>,
) {
    for (cam, mut transform) in query.iter_mut() {
        let mut dir = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) { dir += *transform.forward(); }
        if keys.pressed(KeyCode::KeyS) { dir += *transform.back(); }
        if keys.pressed(KeyCode::KeyA) { dir += *transform.left(); }
        if keys.pressed(KeyCode::KeyD) { dir += *transform.right(); }
        if keys.pressed(KeyCode::Space) { dir += Vec3::Y; }
        if keys.pressed(KeyCode::ShiftLeft) { dir -= Vec3::Y; }
        if dir != Vec3::ZERO {
            transform.translation += dir.normalize() * cam.speed * time.delta_secs();
        }
    }
}

fn scroll_speed(
    scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut SpectatorCamera, With<SpectatorActive>>,
) {
    if scroll.delta.y == 0.0 { return; }
    for mut cam in query.iter_mut() {
        cam.speed = (cam.speed + scroll.delta.y * 2.0).clamp(1.0, 200.0);
    }
}