use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    hit_points::HitPoints,
    movement::{MovingProgress, Speed},
    pathfinding::{enemy_cost_fn, heuristic, NeighborCostIter, PathState},
    tilemap::{AtlasHandle, TilePos, Tilemap},
    GameState,
};
use pathfinding::prelude::astar;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>().add_systems(
            Update,
            (spawn, pathfinding).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Component, Default, Deserialize, Copy, Clone)]
pub enum EnemyKind {
    #[default]
    Skeleton,
    Ent,
}
impl EnemyKind {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Skeleton => 103 * 9 + 36,
            Self::Ent => 103 * 15 + 38,
        }
    }
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
    moving_animation: MovingProgress,
    speed: Speed,
}

fn spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnEnemyEvent>,
    atlas_handle: Res<AtlasHandle>,
    tilemap_query: Query<&Tilemap>,
) {
    for event in events.read() {
        let Ok(tilemap) = tilemap_query.get_single() else {
            continue;
        };

        let world = tilemap.pos_to_world(event.pos);

        commands.spawn((
            EnemyBundle {
                sheet: SpriteSheetBundle {
                    texture_atlas: atlas_handle.0.clone(),
                    sprite: TextureAtlasSprite::new(event.kind.atlas_index()),
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
            },
            #[cfg(feature = "inspector")]
            Name::new("Enemy"),
        ));
    }
}

fn pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePos), (With<Enemy>, Without<PathState>)>,
    tilemap_query: Query<&Tilemap>,
) {
    let Ok(map) = tilemap_query.get_single() else {
        return;
    };

    // TODO spawner should do the pathfinding and cache the result.

    for (entity, pos) in &query {
        let goal = TilePos { x: 61, y: 30 };

        let Some(result) = astar(
            pos,
            |p| NeighborCostIter::new(*p, enemy_cost_fn(&map)),
            |p| heuristic(*p, goal),
            |p| NeighborCostIter::new(goal, enemy_cost_fn(&map)).any(|n| n.0 == *p),
        ) else {
            warn!("Enemy unable to find path to goal.");
            continue;
        };

        commands.entity(entity).insert(PathState::from(result.0));
    }
}
