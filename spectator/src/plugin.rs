use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

pub struct SpectatorPlugin;

#[derive(Component)]
pub struct SpectatorCamera {
    pub speed:       f32,
    pub sensitivity: f32,
    pub yaw:         f32,
    pub pitch:       f32,
}

impl Default for SpectatorCamera {
    fn default() -> Self {
        Self {
            speed:       20.0,
            sensitivity: 0.1,
            yaw:         0.0,
            pitch:       0.0,
        }
    }
}

impl Plugin for SpectatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, grab_cursor)
           .add_systems(Update, mouse_look)
           .add_systems(Update, movement.after(mouse_look));
    }
}

fn grab_cursor(mut cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible   = false;
}

fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut SpectatorCamera, &mut Transform)>,
) {
    let delta = motion.delta;
    if delta == Vec2::ZERO { return; }

    for (mut spec, mut transform) in query.iter_mut() {
        spec.yaw   -= delta.x * spec.sensitivity;
        spec.pitch  = (spec.pitch - delta.y * spec.sensitivity).clamp(-89.0, 89.0);

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            spec.yaw.to_radians(),
            spec.pitch.to_radians(),
            0.0,
        );
    }
}

fn movement(
    time:  Res<Time>,
    keys:  Res<ButtonInput<KeyCode>>,
    mut query: Query<(&SpectatorCamera, &mut Transform)>,
) {
    for (spec, mut transform) in query.iter_mut() {
        let mut dir = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW)     { dir += *transform.forward(); }
        if keys.pressed(KeyCode::KeyS)     { dir += *transform.back(); }
        if keys.pressed(KeyCode::KeyA)     { dir += *transform.left(); }
        if keys.pressed(KeyCode::KeyD)     { dir += *transform.right(); }
        if keys.pressed(KeyCode::Space)    { dir += Vec3::Y; }
        if keys.pressed(KeyCode::ShiftLeft){ dir -= Vec3::Y; }

        if dir != Vec3::ZERO {
            transform.translation += dir.normalize() * spec.speed * time.delta_secs();
        }
    }
}