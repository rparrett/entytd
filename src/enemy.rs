use bevy::prelude::*;
use rand::{seq::SliceRandom, thread_rng};
use serde::Deserialize;

use crate::{
    hit_points::HitPoints,
    home::Home,
    movement::{MovingProgress, Speed},
    particle::{ParticleBundle, ParticleKind, ParticleSettings},
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
            (spawn, pathfinding, behavior, tick_cooldown, attack, die)
                .run_if(in_state(GameState::Playing)),
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
            Self::Ent => 103 * 15 + 45,
        }
    }
}

#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub kind: EnemyKind,
    pub pos: TilePos,
    pub hp: u32,
}

#[derive(Component, Default)]
enum Behavior {
    #[default]
    SeekHome,
    Attack,
}

#[derive(Component)]
pub struct AttackCooldown(Timer);
impl Default for AttackCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Once))
    }
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
    attack_cooldown: AttackCooldown,
    behavior: Behavior,
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
                speed: Speed(2.),
                ..default()
            },
            #[cfg(feature = "inspector")]
            Name::new("Enemy"),
        ));
    }
}

fn pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePos, &Behavior, &EnemyKind), (With<Enemy>, Without<PathState>)>,
    tilemap_query: Query<&Tilemap>,
    home_query: Query<(&TilePos, &HitPoints), With<Home>>,
) {
    // TODO spawner should do the pathfinding and cache the result.
    // TODO i would like for enemies to randomly choose a neighbor of
    // the goal to park in.

    for (entity, pos, behavior, kind) in &query {
        if !matches!(behavior, Behavior::SeekHome) {
            continue;
        }

        let Ok(map) = tilemap_query.get_single() else {
            return;
        };

        let mut rng = thread_rng();
        let goals = home_query
            .iter()
            .filter(|(_, hp)| !hp.is_zero())
            .collect::<Vec<_>>();
        let Some((goal, _)) = goals.choose(&mut rng) else {
            return;
        };

        // choose a random neighbor of the goal and path directly to it,
        // so when enemies are attacking it feels a bit swarmier.
        let neighbors =
            NeighborCostIter::new(**goal, enemy_cost_fn(map, *kind)).collect::<Vec<_>>();
        let Some((goal, _)) = neighbors.choose(&mut rng) else {
            return;
        };

        let Some(result) = astar(
            pos,
            |p| NeighborCostIter::new(*p, enemy_cost_fn(map, *kind)),
            |p| heuristic(*p, *goal),
            |p| *p == *goal,
        ) else {
            warn!("Enemy unable to find path to goal.");
            continue;
        };

        commands.entity(entity).insert(PathState::from(result.0));

        // limit the amount of pathfinding we do each frame.
        break;
    }
}

fn behavior(
    mut removed: RemovedComponents<PathState>,
    mut query: Query<&mut Behavior, (With<Enemy>, Without<PathState>)>,
) {
    // just assume that we've reached the home whenever a PathState is removed.

    let mut enemy_iter = query.iter_many_mut(removed.read());
    while let Some(mut behavior) = enemy_iter.fetch_next() {
        if matches!(*behavior, Behavior::SeekHome) {
            *behavior = Behavior::Attack;
        }
    }
}

fn attack(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Behavior, &mut AttackCooldown, &TilePos),
        (With<Enemy>, Without<PathState>),
    >,
    mut home_query: Query<(&mut HitPoints, &TilePos), With<Home>>,
    tilemap_query: Query<&Tilemap>,
    particle_settings: Res<ParticleSettings>,
) {
    for (entity, behavior, mut cooldown, pos) in &mut query {
        if !matches!(behavior, Behavior::Attack) {
            continue;
        }

        if !cooldown.0.finished() {
            continue;
        }

        let Some((mut home_hp, home_pos)) = home_query
            .iter_mut()
            .find(|(_, home_pos)| heuristic(**home_pos, *pos) == 1)
        else {
            info!("Enemy could not locate a nearby home.");
            continue;
        };

        if home_hp.is_zero() {
            continue;
        }

        let Ok(map) = tilemap_query.get_single() else {
            continue;
        };

        home_hp.sub(1);

        let amt = if home_hp.is_zero() {
            particle_settings.kill_amt()
        } else {
            particle_settings.hit_amt()
        };
        for _ in 0..amt {
            commands.spawn(ParticleBundle::new(
                ParticleKind::Home,
                map.pos_to_world(*home_pos),
            ));
        }

        if home_hp.is_zero() {
            commands.entity(entity).insert(Behavior::SeekHome);
        }

        cooldown.0.reset();
    }
}

fn die(mut commands: Commands, query: Query<(Entity, &HitPoints), With<Enemy>>) {
    for (entity, hp) in &query {
        if !hp.is_zero() {
            continue;
        }

        // TODO particle spam

        commands.entity(entity).despawn();
    }
}

fn tick_cooldown(mut query: Query<&mut AttackCooldown>, time: Res<Time>) {
    for mut cooldown in &mut query {
        cooldown.0.tick(time.delta());
    }
}
