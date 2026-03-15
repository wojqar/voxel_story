use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

const PITCH_DEG: f32 = 45.0;

#[derive(Component)]
pub struct RtsCamera {
    pub pivot: Vec3,
    pub yaw: f32,
    pub zoom: f32,
    pub pan_speed: f32,
    pub rotate_speed: f32,
    pub zoom_speed: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for RtsCamera {
    fn default() -> Self {
        Self {
            pivot: Vec3::ZERO,
            yaw: 0.0,
            zoom: 80.0,
            pan_speed: 40.0,
            rotate_speed: 90.0,
            zoom_speed: 8.0,
            min_zoom: 15.0,
            max_zoom: 250.0,
        }
    }
}

/// Obecność tego komponentu oznacza że RTS camera jest aktywna.
/// Dodawany/usuwany przez switching system w main.rs.
#[derive(Component)]
pub struct RtsActive;

pub struct RtsCameraPlugin;

impl Plugin for RtsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, rts_pan)
            .add_systems(Update, rts_rotate)
            .add_systems(Update, rts_zoom)
            .add_systems(Update, rts_apply_transform
                .after(rts_pan)
                .after(rts_rotate)
                .after(rts_zoom));
    }
}

fn rts_pan(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    for mut cam in query.iter_mut() {
        let yaw_rad = cam.yaw.to_radians();
        let forward = Vec3::new(-yaw_rad.sin(), 0.0, -yaw_rad.cos());
        let right   = Vec3::new( yaw_rad.cos(), 0.0, -yaw_rad.sin());

        let mut delta = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) { delta += forward; }
        if keys.pressed(KeyCode::KeyS) { delta -= forward; }
        if keys.pressed(KeyCode::KeyD) { delta += right; }
        if keys.pressed(KeyCode::KeyA) { delta -= right; }

        if delta != Vec3::ZERO {
            let speed = cam.pan_speed * (cam.zoom / 80.0);
            cam.pivot += delta.normalize() * speed * time.delta_secs();
        }
    }
}

fn rts_rotate(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    for mut cam in query.iter_mut() {
        if keys.pressed(KeyCode::KeyQ) {
            cam.yaw -= cam.rotate_speed * time.delta_secs();
        }
        if keys.pressed(KeyCode::KeyE) {
            cam.yaw += cam.rotate_speed * time.delta_secs();
        }
    }
}

fn rts_zoom(
    scroll: Res<AccumulatedMouseScroll>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    if scroll.delta.y == 0.0 { return; }
    for mut cam in query.iter_mut() {
        cam.zoom = (cam.zoom - scroll.delta.y * cam.zoom_speed)
            .clamp(cam.min_zoom, cam.max_zoom);
    }
}

fn rts_apply_transform(
    mut query: Query<(&RtsCamera, &mut Transform), With<RtsActive>>,
) {
    for (cam, mut transform) in query.iter_mut() {
        let pitch_rad = PITCH_DEG.to_radians();
        let yaw_rad   = cam.yaw.to_radians();

        let local_offset = Vec3::new(
            0.0,
            cam.zoom * pitch_rad.sin(),
            cam.zoom * pitch_rad.cos(),
        );

        let offset = Quat::from_rotation_y(yaw_rad) * local_offset;
        transform.translation = cam.pivot + offset;
        transform.look_at(cam.pivot, Vec3::Y);
    }
}