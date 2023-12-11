use bevy::{prelude::*, window::PrimaryWindow};

use crate::{tilemap::Tilemap, GameState};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn);
        app.add_systems(Update, update.run_if(in_state(GameState::Playing)));
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn update(
    keys: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    tilemaps: Res<Assets<Tilemap>>,
    tilemap_query: Query<&Handle<Tilemap>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let x = keys.any_pressed([KeyCode::Right, KeyCode::D]) as i8
        - keys.any_pressed([KeyCode::Left, KeyCode::A, KeyCode::Q]) as i8;
    let y = keys.any_pressed([KeyCode::Up, KeyCode::W, KeyCode::Z]) as i8
        - keys.any_pressed([KeyCode::Down, KeyCode::S]) as i8;
    let dir = Vec2::new(x as f32, y as f32).normalize_or_zero();

    if dir == Vec2::ZERO {
        return;
    }

    let Ok(mut camera) = query.get_single_mut() else {
        return;
    };

    let Ok(window) = window_query.get_single() else {
        return;
    };

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

    camera.translation += dir.extend(0.) * time.delta_seconds() * speed;

    let max = Vec2::new(tilemap.width as f32, tilemap.height as f32)
        * crate::tilemap::SCALE
        * crate::tilemap::TILE_SIZE
        / 2.
        - Vec2::new(window.width(), window.height()) / 2.;
    let min = -max;

    camera.translation.x = camera.translation.x.clamp(min.x, max.x);
    camera.translation.y = camera.translation.y.clamp(min.y, max.y);
}
