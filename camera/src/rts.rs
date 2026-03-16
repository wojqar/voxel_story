use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use world_api::{TerrainHeightRequest, TerrainHeightResponse};

const PITCH_DEG: f32 = 45.0;

pub struct RtsCameraPlugin;

impl Plugin for RtsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerrainHeightRequest>()
            .add_message::<TerrainHeightResponse>()
            .add_systems(Update, rts_pan.run_if(any_with_component::<RtsActive>))
            .add_systems(Update, rts_rotate.run_if(any_with_component::<RtsActive>))
            .add_systems(Update, rts_zoom.run_if(any_with_component::<RtsActive>))
            .add_systems(Update, rts_pivot_y
                .after(rts_pan)
                .run_if(any_with_component::<RtsActive>))
            .add_systems(Update, rts_apply_transform
                .after(rts_pivot_y)
                .run_if(any_with_component::<RtsActive>));
    }
}

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
    pub pivot_y_target: f32,
    pub pivot_y_lerp_speed: f32,
    pub terrain_snap: bool,
    pub manual_y_speed: f32,
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
            pivot_y_target: 0.0,
            pivot_y_lerp_speed: 8.0,
            terrain_snap: true,
            manual_y_speed: 20.0,
        }
    }
}

#[derive(Component)]
pub struct RtsActive;

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
            cam.terrain_snap = true;
        }
    }
}

fn rts_rotate(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    for mut cam in query.iter_mut() {
        if keys.pressed(KeyCode::KeyQ) { cam.yaw -= cam.rotate_speed * time.delta_secs(); }
        if keys.pressed(KeyCode::KeyE) { cam.yaw += cam.rotate_speed * time.delta_secs(); }
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

fn rts_pivot_y(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut requests: MessageWriter<TerrainHeightRequest>,
    mut responses: MessageReader<TerrainHeightResponse>,
    mut query: Query<&mut RtsCamera, With<RtsActive>>,
) {
    for mut cam in query.iter_mut() {
        if keys.pressed(KeyCode::PageUp) {
            cam.pivot.y += cam.manual_y_speed * time.delta_secs();
            cam.terrain_snap = false;
        } else if keys.pressed(KeyCode::PageDown) {
            cam.pivot.y -= cam.manual_y_speed * time.delta_secs();
            cam.terrain_snap = false;
        }

        if !cam.terrain_snap { continue; }

        requests.write(TerrainHeightRequest {
            pos: Vec2::new(cam.pivot.x, cam.pivot.z),
        });

        for response in responses.read() {
            cam.pivot_y_target = response.height;
        }

        let t = (cam.pivot_y_lerp_speed * time.delta_secs()).min(1.0);
        cam.pivot.y += (cam.pivot_y_target - cam.pivot.y) * t;
    }
}

fn rts_apply_transform(
    mut query: Query<(&RtsCamera, &mut Transform), With<RtsActive>>,
) {
    for (cam, mut transform) in query.iter_mut() {
        let pitch_rad = PITCH_DEG.to_radians();
        let yaw_rad   = cam.yaw.to_radians();
        let offset = Quat::from_rotation_y(yaw_rad)
            * Vec3::new(0.0, cam.zoom * pitch_rad.sin(), cam.zoom * pitch_rad.cos());
        transform.translation = cam.pivot + offset;
        transform.look_at(cam.pivot, Vec3::Y);
    }
}