use bevy::prelude::*;
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use serde::Deserialize;

use crate::{
    hit_points::HitPoints,
    home::Home,
    movement::{MovingProgress, Speed},
    particle::{ParticleBundle, ParticleKind},
    pathfinding::{enemy_cost_fn, heuristic, NeighborCostIter, PathState},
    settings::{DifficultySetting, ParticlesSetting},
    stats::Stats,
    tilemap::{AtlasHandle, Map, TilePos},
    util::cleanup,
    GameState,
};
use pathfinding::prelude::astar;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>()
            .init_resource::<EnemyRng>()
            .add_systems(
                Update,
                (spawn, pathfinding, behavior, tick_cooldown, attack, die)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup::<Enemy>);
    }
}

#[derive(Component, Default)]
pub struct Enemy;

#[derive(Component, Default, Deserialize, Copy, Clone)]
pub enum EnemyKind {
    #[default]
    Skeleton,
    SkeletonTwo,
    SkeletonThree,
    SkeletonFour,
    Ent,
    EntTwo,
    EntThree,
    EntFour,
}
impl EnemyKind {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Skeleton => 103 * 9 + 36,
            Self::SkeletonTwo => 103 * 9 + 37,
            Self::SkeletonThree => 103 * 9 + 38,
            Self::SkeletonFour => 103 * 9 + 42,
            Self::Ent => 103 * 15 + 45,
            Self::EntTwo => 103 * 15 + 52,
            Self::EntThree => 103 * 15 + 53,
            Self::EntFour => 103 * 15 + 54,
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

#[derive(Resource)]
pub struct EnemyRng(SmallRng);
impl Default for EnemyRng {
    fn default() -> Self {
        Self(SmallRng::from_entropy())
    }
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    sprite: SpriteBundle,
    texture: TextureAtlas,
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
    tilemap_query: Query<&Map>,
    difficulty: Res<DifficultySetting>,
) {
    for event in events.read() {
        let Ok(tilemap) = tilemap_query.get_single() else {
            continue;
        };

        let world = tilemap.pos_to_world(event.pos);

        let hp = match *difficulty {
            DifficultySetting::Hard => event.hp,
            DifficultySetting::Normal => ((event.hp as f32 * 0.75).floor() as u32).max(1),
            DifficultySetting::Impossible => ((event.hp as f32 * 1.25).floor() as u32).max(1),
        };

        commands.spawn((
            EnemyBundle {
                sprite: SpriteBundle {
                    texture: atlas_handle.image.clone(),
                    transform: Transform {
                        translation: world.extend(1.),
                        scale: crate::tilemap::SCALE.extend(1.),
                        ..default()
                    },
                    ..default()
                },
                texture: TextureAtlas {
                    layout: atlas_handle.layout.clone(),
                    index: event.kind.atlas_index(),
                },
                hit_points: HitPoints::full(hp),
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
    tilemap_query: Query<&Map>,
    home_query: Query<(&TilePos, &HitPoints), With<Home>>,
    mut rng: ResMut<EnemyRng>,
) {
    // TODO spawner should do the pathfinding and cache the result.

    for (entity, pos, behavior, kind) in &query {
        if !matches!(behavior, Behavior::SeekHome) {
            continue;
        }

        let Ok(map) = tilemap_query.get_single() else {
            return;
        };

        let goals = home_query
            .iter()
            .filter(|(_, hp)| !hp.is_zero())
            .collect::<Vec<_>>();
        let Some((goal, _)) = goals.choose(&mut rng.0) else {
            return;
        };

        // choose a random neighbor of the goal and path directly to it,
        // so when enemies are attacking it feels a bit swarmier.
        let neighbors =
            NeighborCostIter::new(**goal, enemy_cost_fn(map, *kind)).collect::<Vec<_>>();
        let Some((goal, _)) = neighbors.choose(&mut rng.0) else {
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

        // The enemy may have died and been despawned in the same frame.
        commands
            .entity(entity)
            .try_insert(PathState::from(result.0));

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
    tilemap_query: Query<&Map>,
    particle_settings: Res<ParticlesSetting>,
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
            warn!("Enemy could not locate a nearby home.");
            continue;
        };

        if home_hp.is_zero() {
            // Enemy may have died and been despawned
            commands.entity(entity).try_insert(Behavior::SeekHome);
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
            // Enemy may have died and been despawned
            commands.entity(entity).try_insert(Behavior::SeekHome);
        }

        cooldown.0.reset();
    }
}

fn die(
    mut commands: Commands,
    query: Query<(Entity, &HitPoints), With<Enemy>>,
    mut stats: ResMut<Stats>,
) {
    for (entity, hp) in &query {
        if !hp.is_zero() {
            continue;
        }

        stats.kills += 1;

        commands.entity(entity).despawn();
    }
}

fn tick_cooldown(mut query: Query<&mut AttackCooldown>, time: Res<Time>) {
    for mut cooldown in &mut query {
        cooldown.0.tick(time.delta());
    }
}
