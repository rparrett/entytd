use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{tilemap::Map, GameState};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(Update, update.run_if(in_state(GameState::Playing)));
    }
}

fn spawn(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedHorizontal(1280.);
    commands.spawn(camera);
}

pub fn update(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&OrthographicProjection, &mut Transform), With<Camera2d>>,
    time: Res<Time>,
    tilemaps: Res<Assets<Map>>,
    tilemap_query: Query<&Handle<Map>>,
    window_query: Query<&Window>,
) {
    let Ok((projection, mut transform)) = query.get_single_mut() else {
        return;
    };

    let min_speed = 250.;
    let max_speed = 500.;

    let x = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) as i8
        - keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) as i8;
    let y = keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) as i8
        - keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) as i8;
    let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

    if dir != Vec2::ZERO {
        let speed = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            max_speed
        } else {
            min_speed
        };

        transform.translation += dir.extend(0.) * time.delta_seconds() * speed;
    }

    let Ok(window) = window_query.get_single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        fn map_range(from: (f32, f32), to: (f32, f32), value: f32) -> f32 {
            to.0 + (value - from.0) * (to.1 - to.0) / (from.1 - from.0)
        }

        let cursor_position = Vec2::new(cursor_position.x, cursor_position.y);
        let half_viewport_size = window.resolution.size() / 2.;
        let center_to_cursor = cursor_position - half_viewport_size;
        let normalized_length = center_to_cursor / half_viewport_size;

        if normalized_length.x.abs() >= 0.8 {
            transform.translation.x += time.delta_seconds()
                * map_range((0.8, 1.), (min_speed, max_speed), normalized_length.x.abs())
                    .copysign(normalized_length.x);
        }
        if normalized_length.y.abs() >= 0.8 {
            transform.translation.y -= time.delta_seconds()
                * map_range((0.8, 1.), (min_speed, max_speed), normalized_length.y.abs())
                    .copysign(normalized_length.y);
        }
    }

    let Ok(tilemap_handle) = tilemap_query.get_single() else {
        return;
    };

    let Some(tilemap) = tilemaps.get(tilemap_handle) else {
        return;
    };

    let pan_area = tilemap.size_vec2() * crate::tilemap::SCALE * crate::tilemap::TILE_SIZE
        - projection.area.size();

    if pan_area.x <= 0. || pan_area.y <= 0. {
        return;
    }

    let min = pan_area / -2.;
    let max = pan_area / 2.;

    transform.translation.x = transform.translation.x.clamp(min.x, max.x);
    transform.translation.y = transform.translation.y.clamp(min.y, max.y);
}
