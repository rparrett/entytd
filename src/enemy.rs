use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    hit_points::HitPoints,
    movement::{MovingProgress, Speed},
    pathfinding::{EnemyPathfinding, PathState},
    tilemap::{AtlasHandle, TilePos, Tilemap, TilemapHandle},
    GameState,
};

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
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    for event in events.read() {
        let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
            continue;
        };

        let world = tilemap.pos_to_world(event.pos);

        commands.spawn((
            EnemyBundle {
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
            },
            #[cfg(feature = "inspector")]
            Name::new("Enemy"),
        ));
    }
}

fn pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePos), (With<Enemy>, Without<PathState>)>,
    pathfinding: Option<Res<EnemyPathfinding>>,
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
        let Some(path) = pathfinding.0.find_path(
            (pos.x, pos.y),
            (62, 30),
            crate::pathfinding::enemy_cost_fn(&map),
        ) else {
            warn!("Enemy unable to find path to goal.");
            continue;
        };

        let resolved = path.resolve(crate::pathfinding::enemy_cost_fn(&map));

        commands.entity(entity).insert(PathState::from(resolved));
    }
}
