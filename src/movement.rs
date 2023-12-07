use bevy::prelude::*;

use crate::{
    pathfinding::PathState,
    tilemap::{TilePos, Tilemap},
    GameState,
};

/// This is the speed in "tiles lengths per second."
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
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut TilePos,
        &mut PathState,
        &mut MovingProgress,
        &Speed,
    )>,
    tilemap_query: Query<&Tilemap>,
    time: Res<Time>,
) {
    let Ok(map) = tilemap_query.get_single() else {
        return;
    };

    for (entity, mut transform, mut tile_pos, mut path_state, mut animation, speed) in &mut query {
        if path_state.finished() {
            commands.entity(entity).remove::<PathState>();
            continue;
        }

        let mut current = path_state.path[path_state.index];
        let mut next = path_state.path[path_state.index + 1];

        animation.0 += time.delta_seconds() * speed.0;

        while animation.0 > 1.0 {
            path_state.index += 1;

            if !path_state.finished() {
                current = path_state.path[path_state.index];
                next = path_state.path[path_state.index + 1];

                animation.0 -= 1.0;
            } else {
                animation.0 = 1.0;
                break;
            }
        }

        let current_world = map.pos_to_world(current);
        let next_world = map.pos_to_world(next);

        let diff = next_world - current_world;
        let step = diff * animation.0;

        let lerped = current_world + step;

        transform.translation.x = lerped.x;
        transform.translation.y = lerped.y;

        if path_state.finished() {
            *tile_pos = next;
            commands.entity(entity).remove::<PathState>();
        } else {
            *tile_pos = current;
        }
    }
}
