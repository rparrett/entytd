use bevy::{prelude::*, render::camera::ScalingMode};

use crate::{tilemap::Tilemap, GameState};

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
    mut query: Query<(Ref<OrthographicProjection>, &mut Transform), With<Camera2d>>,
    time: Res<Time>,
    tilemaps: Res<Assets<Tilemap>>,
    tilemap_query: Query<&Handle<Tilemap>>,
) {
    let x = keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) as i8
        - keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) as i8;
    let y = keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) as i8
        - keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) as i8;
    let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

    let Ok((projection, mut transform)) = query.get_single_mut() else {
        return;
    };

    if dir == Vec2::ZERO && !projection.is_changed() {
        return;
    }

    let Ok(tilemap_handle) = tilemap_query.get_single() else {
        return;
    };

    let Some(tilemap) = tilemaps.get(tilemap_handle) else {
        return;
    };

    let speed = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        500.
    } else {
        250.
    };

    let pan_area = Vec2::new(tilemap.width as f32, tilemap.height as f32)
        * crate::tilemap::SCALE
        * crate::tilemap::TILE_SIZE
        - projection.area.size();

    if pan_area.x <= 0. || pan_area.y <= 0. {
        return;
    }

    let min = pan_area / -2.;
    let max = pan_area / 2.;

    transform.translation += dir.extend(0.) * time.delta_seconds() * speed;
    transform.translation.x = transform.translation.x.clamp(min.x, max.x);
    transform.translation.y = transform.translation.y.clamp(min.y, max.y);
}
