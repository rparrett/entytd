use crate::{
    designate_tool::Designations,
    hit_points::HitPoints,
    movement::{MovingProgress, Speed},
    pathfinding::{heuristic, worker_cost_fn, NeighborCostIter, PathState},
    tilemap::{AtlasHandle, TileEntities, TileKind, TilePos, Tilemap, TilemapHandle},
    GameState,
};
use bevy::prelude::*;
use pathfinding::prelude::astar;
use rand::{seq::SliceRandom, thread_rng, Rng};

pub struct WorkerPlugin;
impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnWorkerEvent>()
            .add_systems(Update, spawn.run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(
                Update,
                (find_job, do_job, tick_cooldown).run_if(in_state(GameState::Playing)),
            );
    }
}

const WORKER_SPRITES: [usize; 2] = [103 * 14 + 0, 103 * 15 + 0];

#[derive(Component, Default)]
pub struct Worker;

#[derive(Component)]
pub struct Idle;

#[derive(Component)]
pub enum Job {
    Dig(TilePos),
}

#[derive(Component)]
pub struct WorkCooldown(Timer);
impl Default for WorkCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Once))
    }
}

#[derive(Bundle, Default)]
pub struct WorkerBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    worker: Worker,
    pos: TilePos,
    moving_animation: MovingProgress,
    speed: Speed,
    work_cooldown: WorkCooldown,
}

#[derive(Event)]
pub struct SpawnWorkerEvent;

fn spawn(
    mut commands: Commands,
    mut events: EventReader<SpawnWorkerEvent>,
    atlas_handle: Res<AtlasHandle>,
    tilemap_query: Query<&Tilemap>,
) {
    if events.is_empty() {
        return;
    }

    let mut rng = thread_rng();

    for _ in events.read() {
        let Ok(tilemap) = tilemap_query.get_single() else {
            continue;
        };

        let index = *WORKER_SPRITES.choose(&mut rng).unwrap();
        let color = Color::hsl(rng.gen_range(0.0..=360.0), 0.9, 0.5);

        let home = (60, 30);
        let pos = TilePos {
            x: rng.gen_range((home.0 - 2)..(home.0 + 2)),
            y: rng.gen_range((home.1 - 2)..(home.1 + 2)),
        };
        let world = tilemap.pos_to_world(pos);

        commands.spawn((
            WorkerBundle {
                sheet: SpriteSheetBundle {
                    texture_atlas: atlas_handle.0.clone(),
                    sprite: TextureAtlasSprite {
                        index,
                        color,
                        ..default()
                    },
                    transform: Transform {
                        translation: world.extend(1.),
                        scale: crate::tilemap::SCALE.extend(1.),
                        ..default()
                    },
                    ..default()
                },
                hit_points: HitPoints::full(2),
                pos,
                ..default()
            },
            Idle,
            #[cfg(feature = "inspector")]
            Name::new("Worker"),
        ));
    }
}

fn init(mut events: EventWriter<SpawnWorkerEvent>) {
    for _ in 0..10 {
        events.send(SpawnWorkerEvent);
    }
}

fn find_job(
    mut commands: Commands,
    query: Query<(Entity, &TilePos), (With<Worker>, With<Idle>, Without<PathState>)>,
    mut designations: ResMut<Designations>,
    tilemap_query: Query<&Tilemap>,
) {
    let Ok(map) = tilemap_query.get_single() else {
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
            if NeighborCostIter::new(*pos, worker_cost_fn(&map))
                .next()
                .is_none()
            {
                return None;
            }

            Some((*pos, designation.kind))
        })
        .collect::<Vec<_>>();

    if potential_jobs.is_empty() {
        return;
    }

    let mut jobs_assigned = vec![];

    for (entity, pos) in &query {
        let now = std::time::Instant::now();

        potential_jobs.sort_by_key(|a| u32::MAX - heuristic(a.0, *pos));

        let Some((goal, designation)) = potential_jobs.pop() else {
            continue;
        };

        let Some(result) = astar(
            pos,
            |p| NeighborCostIter::new(*p, worker_cost_fn(&map)),
            |p| heuristic(*p, goal),
            |p| NeighborCostIter::new(goal, worker_cost_fn(&map)).any(|n| n.0 == *p),
        ) else {
            warn!(
                "Worker unable to find path to goal. ({}ms)",
                now.elapsed().as_secs_f32() * 1000.
            );
            continue;
        };

        commands
            .entity(entity)
            .insert(MovingProgress::default())
            .insert(PathState::from(result.0))
            .insert(Job::Dig(goal))
            .remove::<Idle>();

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
        (Entity, &TilePos, &Job, &mut WorkCooldown),
        (With<Worker>, Without<Idle>, Without<PathState>),
    >,
    mut dig_query: Query<&mut HitPoints>,
    mut tilemap_query: Query<(&mut Tilemap, &mut TileEntities)>,
    mut designations: ResMut<Designations>,
    mut tile_query: Query<(&mut TileKind, &mut TextureAtlasSprite)>,
) {
    if query.is_empty() {
        return;
    }

    let Ok((mut map, map_entities)) = tilemap_query.get_single_mut() else {
        return;
    };

    for (entity, _pos, job, mut cooldown) in &mut query {
        // TODO ensure we are actually near the job.

        if !cooldown.0.finished() {
            continue;
        }

        let mut did_work = false;

        match job {
            Job::Dig(dig_pos) => {
                let Some(tile_entity) = map_entities.entities[dig_pos.x][dig_pos.y] else {
                    commands.entity(entity).insert(Idle);
                    commands.entity(entity).remove::<Job>();
                    warn!("Working trying to dig at position without entity.");
                    continue;
                };

                let Ok(mut hp) = dig_query.get_mut(tile_entity) else {
                    commands.entity(entity).insert(Idle);
                    commands.entity(entity).remove::<Job>();
                    warn!("Worker trying dig entity without hitpoints.");
                    continue;
                };

                hp.sub(1);

                if hp.is_zero() {
                    commands.entity(entity).insert(Idle);
                    commands.entity(entity).remove::<Job>();

                    // // TODO RemoveDesignationEvent?

                    if let Some(designation) = designations.0.remove(dig_pos) {
                        commands.entity(designation.indicator).despawn();
                    }

                    map.tiles[dig_pos.x][dig_pos.y] = TileKind::Dirt;

                    let Some(tile_entity) = map_entities.entities[dig_pos.x][dig_pos.y] else {
                        continue;
                    };
                    let Ok((mut kind, mut sprite)) = tile_query.get_mut(tile_entity) else {
                        continue;
                    };
                    *kind = TileKind::Dirt;
                    sprite.index = kind.atlas_index();
                }

                did_work = true;
            }
        }

        if did_work {
            info!("Resetting cooldown.");
            cooldown.0.reset();
        }
    }
}

fn tick_cooldown(mut query: Query<&mut WorkCooldown>, time: Res<Time>) {
    for mut cooldown in &mut query {
        cooldown.0.tick(time.delta());
    }
}
