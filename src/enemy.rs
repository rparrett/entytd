use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    hit_points::HitPoints,
    pathfinding::{PathState, Pathfinding},
    tilemap::{AtlasHandle, TilePos, Tilemap, TilemapHandle},
    GameState,
};

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>().add_systems(
            Update,
            (spawn, pathfinding, movement).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Component, Default, Deserialize, Copy, Clone)]
pub enum EnemyKind {
    #[default]
    Skeleton,
}

#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub kind: EnemyKind,
    pub pos: TilePos,
    pub hp: u32,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    enemy: Enemy,
    kind: EnemyKind,
    pos: TilePos,
    moving_animation: MovingAnimation,
}

#[derive(Component, Default)]
struct MovingAnimation(f32);

fn spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnEnemyEvent>,
    atlas_handle: Res<AtlasHandle>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    for event in events.read() {
        let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
            continue;
        };

        let world = tilemap.pos_to_world(event.pos);

        commands.spawn(EnemyBundle {
            sheet: SpriteSheetBundle {
                texture_atlas: atlas_handle.0.clone(),
                sprite: TextureAtlasSprite::new(103 * 9 + 36),
                transform: Transform {
                    translation: world.extend(1.),
                    scale: crate::tilemap::SCALE.extend(1.),
                    ..default()
                },
                ..default()
            },
            hit_points: HitPoints::full(event.hp),
            kind: event.kind,
            pos: event.pos,
            ..default()
        });
    }
}

fn pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePos), (With<Enemy>, Without<PathState>)>,
    pathfinding: Option<Res<Pathfinding>>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    let Some(map) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    let Some(pathfinding) = pathfinding else {
        return;
    };

    for (entity, pos) in &query {
        let Some(path) =
            pathfinding
                .0
                .find_path((pos.x, pos.y), (62, 30), crate::pathfinding::cost_fn(&map))
        else {
            warn!("Enemy unable to find path to goal.");
            continue;
        };

        let resolved = path.resolve(crate::pathfinding::cost_fn(&map));

        commands.entity(entity).insert(PathState::from(resolved));
    }
}

fn movement(
    mut query: Query<
        (
            &mut Transform,
            &TilePos,
            &mut PathState,
            &mut MovingAnimation,
        ),
        With<Enemy>,
    >,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
    time: Res<Time>,
) {
    let Some(map) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    for (mut transform, pos, mut path_state, mut animation) in &mut query {
        if path_state.finished() {
            continue;
        }

        let mut current = path_state.path[path_state.index];
        let mut next = path_state.path[path_state.index + 1];

        let mut current_world = map.pos_to_world(current);
        let mut next_world = map.pos_to_world(next);

        let speed = 10.0;
        animation.0 += time.delta_seconds() * speed;

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
