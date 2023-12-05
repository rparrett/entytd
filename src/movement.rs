use bevy::prelude::*;

use crate::{
    pathfinding::PathState,
    tilemap::{TilePos, Tilemap, TilemapHandle},
    GameState,
};

#[derive(Component)]
pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(10.)
    }
}

#[derive(Component, Default)]
pub struct MovingProgress(f32);

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement.run_if(in_state(GameState::Playing)));
    }
}

fn movement(
    mut query: Query<(&mut Transform, &mut PathState, &mut MovingProgress, &Speed)>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
    time: Res<Time>,
) {
    let Some(map) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    for (mut transform, mut path_state, mut animation, speed) in &mut query {
        if path_state.finished() {
            continue;
        }

        let mut current = path_state.path[path_state.index];
        let mut next = path_state.path[path_state.index + 1];

        let mut current_world = map.pos_to_world(current);
        let mut next_world = map.pos_to_world(next);

        animation.0 += time.delta_seconds() * speed.0;

        while animation.0 > 1.0 {
            path_state.index += 1;

            if !path_state.finished() {
                current = path_state.path[path_state.index];
                next = path_state.path[path_state.index + 1];

                current_world = map.pos_to_world(current);
                next_world = map.pos_to_world(next);

                animation.0 -= 1.0;
            } else {
                animation.0 = 1.0;
                break;
            }
        }

        let diff = next_world - current_world;
        let step = diff * animation.0;

        let lerped = current_world + step;

        transform.translation.x = lerped.x;
        transform.translation.y = lerped.y;
    }
}
