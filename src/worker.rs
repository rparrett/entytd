use crate::{
    designate_tool::{DesignationKind, Designations},
    hit_points::HitPoints,
    layer,
    level::{LevelConfig, LevelHandle},
    movement::{MovingProgress, Speed},
    pathfinding::{heuristic, worker_cost_fn, NeighborCostIter, PathState},
    settings::SfxSetting,
    sound::SoundAssets,
    stats::Stats,
    stone::HitStoneEvent,
    tilemap::{AtlasHandle, Map, TileEntities, TileKind, TilePos},
    tower::BuildTowerEvent,
    GameState,
};
use bevy::{audio::Volume, prelude::*};
use pathfinding::prelude::astar;
use rand::{rngs::SmallRng, seq::IndexedRandom, Rng, SeedableRng};

pub struct WorkerPlugin;
impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnWorkerEvent>()
            .init_resource::<WorkerSortTimer>()
            .init_resource::<WorkerRng>()
            .add_systems(Update, spawn.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(
                Update,
                (find_job, do_job, tick_cooldown, sort_workers)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

const WORKER_SPRITES: [usize; 2] = [103 * 14, 103 * 15];

#[derive(Component, Default)]
#[require(Sprite, HitPoints, TilePos, MovingProgress, Speed, WorkCooldown)]
pub struct Worker;

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub enum Job {
    Dig(TilePos),
    Build { hit_points: HitPoints, pos: TilePos },
}

#[derive(Component)]
pub struct WorkCooldown(Timer);
impl Default for WorkCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Once))
    }
}
#[derive(Message)]
pub struct SpawnWorkerEvent;

#[derive(Resource)]
pub struct WorkerSortTimer(Timer);
impl Default for WorkerSortTimer {
    fn default() -> Self {
        WorkerSortTimer(Timer::from_seconds(2., TimerMode::Repeating))
    }
}

#[derive(Resource)]
pub struct WorkerRng(SmallRng);
impl Default for WorkerRng {
    fn default() -> Self {
        Self(SmallRng::from_os_rng())
    }
}

fn spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnWorkerEvent>,
    atlas_handle: Res<AtlasHandle>,
    tilemap_query: Query<&Map>,
    mut rng: ResMut<WorkerRng>,
) {
    if events.is_empty() {
        return;
    }

    for _ in events.read() {
        let Ok(tilemap) = tilemap_query.single() else {
            continue;
        };

        let index = *WORKER_SPRITES.choose(&mut rng.0).unwrap();
        let color = Color::hsl(rng.0.gen_range(0.0..=360.0), 0.9, 0.5);

        let home = (60, 30);
        let pos = TilePos {
            x: rng.0.gen_range((home.0 - 2)..(home.0 + 2)),
            y: rng.0.gen_range((home.1 - 2)..(home.1 + 2)),
        };
        let world = tilemap.pos_to_world(pos);

        commands.spawn((
            Worker,
            Sprite {
                image: atlas_handle.image.clone(),
                color,
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_handle.layout.clone(),
                    index,
                }),
                ..default()
            },
            Transform {
                // Give workers a random z value so their display order is stable as
                // entities are added/removed from the view/world.
                translation: world.extend(layer::MOBS + rng.0.gen::<f32>()),
                scale: crate::tilemap::SCALE.extend(1.),
                ..default()
            },
            HitPoints::full(2),
            pos,
            Idle,
            Name::new("Worker"),
        ));
    }
}

fn init(
    mut events: EventWriter<SpawnWorkerEvent>,
    levels: Res<Assets<LevelConfig>>,
    level_handle: Res<LevelHandle>,
) {
    let Some(level) = levels.get(&level_handle.0) else {
        warn!("Couldn't find level when initializing Currency ");
        return;
    };

    for _ in 0..level.workers {
        events.write(SpawnWorkerEvent);
    }
}

fn find_job(
    mut commands: Commands,
    query: Query<(Entity, &TilePos), (With<Worker>, With<Idle>, Without<PathState>)>,
    mut designations: ResMut<Designations>,
    tilemap_query: Query<&Map>,
) {
    let Ok(map) = tilemap_query.single() else {
        return;
    };

    if query.is_empty() {
        return;
    }

    let mut potential_jobs = designations
        .0
        .iter()
        .filter_map(|(pos, designation)| {
            // filter out jobs that already have enough workers.
            if designation.workers >= 4 {
                return None;
            }

            // filter out jobs that are definitely unreachable because their
            // immediate neighbors are not walkable.
            NeighborCostIter::new(*pos, worker_cost_fn(map)).next()?;

            Some((*pos, designation))
        })
        .collect::<Vec<_>>();

    if potential_jobs.is_empty() {
        return;
    }

    let mut jobs_assigned = vec![];

    for (entity, pos) in &query {
        // Sort higher priority jobs to the end of the array.
        // First, if there's a tower designated with no workers assigned, do that.
        // Then, choose the closest job.
        potential_jobs.sort_by_key(|a| {
            let dist = u32::MAX - heuristic(a.0, *pos);
            let tower_with_no_workers =
                matches!(a.1.kind, DesignationKind::BuildTower) && a.1.workers < 1;

            (tower_with_no_workers, dist)
        });

        let Some((goal, designation)) = potential_jobs.pop() else {
            continue;
        };

        let Some(result) = astar(
            pos,
            |p| NeighborCostIter::new(*p, worker_cost_fn(map)),
            |p| heuristic(*p, goal),
            |p| NeighborCostIter::new(goal, worker_cost_fn(map)).any(|n| n.0 == *p),
        ) else {
            warn!("Worker unable to find path to goal.");
            continue;
        };

        let mut command = commands.entity(entity);

        command.insert(PathState::from(result.0)).remove::<Idle>();

        match designation.kind {
            DesignationKind::Dig => {
                command.insert(Job::Dig(goal));
            }
            DesignationKind::BuildTower => {
                command.insert(Job::Build {
                    hit_points: HitPoints::full(10),
                    pos: goal,
                });
            }
            _ => {}
        }

        jobs_assigned.push(goal);

        // limit the amount of pathfinding we do each frame.
        break;
    }

    for goal in jobs_assigned {
        designations.0.get_mut(&goal).unwrap().workers += 1;
    }
}

fn do_job(
    mut commands: Commands,
    mut query: Query<
        (Entity, &TilePos, &mut Job, &mut WorkCooldown),
        (With<Worker>, Without<Idle>, Without<PathState>),
    >,
    dig_query: Query<&HitPoints>,
    tile_kind_query: Query<&TileKind>,
    mut tilemap_query: Query<&TileEntities>,
    mut events: EventWriter<HitStoneEvent>,
    mut tower_events: EventWriter<BuildTowerEvent>,
    sound_assets: Res<SoundAssets>,
    sfx_setting: Res<SfxSetting>,
    mut stats: ResMut<Stats>,
) {
    if query.is_empty() {
        return;
    }

    let Ok(map_entities) = tilemap_query.single_mut() else {
        return;
    };

    for (entity, _pos, mut job, mut cooldown) in &mut query {
        // TODO ensure we are actually near the job.

        match &mut *job {
            Job::Dig(dig_pos) => {
                let Some(tile_entity) = map_entities.0[(dig_pos.y, dig_pos.x)] else {
                    warn!("Working trying to dig at position without entity.");
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                };

                // TODO maybe also just double check that this is a rock and not just any
                // random thing with hitpoints.

                let Ok(hp) = dig_query.get(tile_entity) else {
                    warn!("Working trying to dig at position without HP.");
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                };

                if hp.is_zero() {
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                }

                if !cooldown.0.finished() {
                    continue;
                }

                events.write(HitStoneEvent {
                    entity: tile_entity,
                    damage: 1,
                });

                commands.spawn((
                    AudioPlayer(sound_assets.pickaxe.clone()),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(**sfx_setting as f32 / 100.)),
                ));

                cooldown.0.reset();
            }
            Job::Build { hit_points, pos } => {
                let Some(tile_entity) = map_entities.0[(pos.y, pos.x)] else {
                    warn!("Working trying to build at position without entity.");
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                };

                // TODO maybe also just double check that this is a rock and not just any
                // random thing with hitpoints.

                let Ok(kind) = tile_kind_query.get(tile_entity) else {
                    warn!("Working trying to build at position without tile kind.");
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                };

                if !kind.buildable() {
                    warn!("Working trying to build at position without tile kind.");
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                }

                if hit_points.is_zero() {
                    commands.entity(entity).insert(Idle).remove::<Job>();
                    continue;
                }

                if !cooldown.0.finished() {
                    continue;
                }

                hit_points.sub(1);

                if hit_points.is_zero() {
                    stats.towers += 1;
                    tower_events.write(BuildTowerEvent(*pos));
                }

                cooldown.0.reset();
            }
        }
    }
}

fn tick_cooldown(mut query: Query<&mut WorkCooldown>, time: Res<Time>) {
    for mut cooldown in &mut query {
        cooldown.0.tick(time.delta());
    }
}

/// Randomize worker z order every couple seconds so that it's possible to
/// tell that there is more than one worker standing on a particular tile.
fn sort_workers(
    mut query: Query<&mut Transform, With<Worker>>,
    mut timer: ResMut<WorkerSortTimer>,
    time: Res<Time>,
    mut rng: ResMut<WorkerRng>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    for mut translation in &mut query {
        translation.translation.z = layer::MOBS + rng.0.gen::<f32>();
    }
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<Worker>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
