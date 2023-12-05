use crate::{
    designate_tool::Designations,
    hit_points::HitPoints,
    movement::{MovingProgress, Speed},
    pathfinding::{PathState, WorkerPathfinding},
    tilemap::{AtlasHandle, TileEntities, TilePos, Tilemap, TilemapHandle},
    GameState,
};
use bevy::prelude::*;
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
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    if events.is_empty() {
        return;
    }

    let mut rng = thread_rng();

    for _ in events.read() {
        let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
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
    query: Query<(Entity, &TilePos), (With<Worker>, With<Idle>)>,
    designations: Res<Designations>,
    pathfinding: Option<Res<WorkerPathfinding>>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    let Some(map) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    let Some(pathfinding) = pathfinding else {
        return;
    };

    if query.is_empty() {
        info!("no idle workers?");
        return;
    }

    let mut designation = None;
    for x in 0..map.width {
        for y in 0..map.height {
            if let Some(d) = &designations.0[x][y] {
                designation = Some((x, y, d));
            }
        }
    }

    let Some((x, y, designation)) = designation else {
        return;
    };

    let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];

    for (entity, pos) in &query {
        let diff: IVec2 =
            (IVec2::new(pos.x as i32, pos.y as i32) - IVec2::new(x as i32, y as i32)).abs();
        if diff.x + diff.y < 1 {
            info!("skipping, already here.");
            continue;
        }

        // TODO find_closest_goal seems ideal here, but it is is sometimes panicking.
        // https://github.com/mich101mich/hierarchical_pathfinding/issues/7

        let Some(path) = neighbors.iter().find_map(|n| {
            pathfinding
                .0
                .find_path((pos.x, pos.y), *n, crate::pathfinding::worker_cost_fn(&map))
        }) else {
            warn!("Worker unable to find path to goal.");
            continue;
        };

        let resolved = path.resolve(crate::pathfinding::worker_cost_fn(&map));

        commands
            .entity(entity)
            .insert(PathState::from(resolved))
            .insert(Job::Dig((x, y).into()))
            .remove::<Idle>();

        // limit the amount of pathfinding we do each frame.
        break;
    }
}

fn do_job(
    mut commands: Commands,
    mut query: Query<
        (Entity, &TilePos, &Job, &mut WorkCooldown),
        (With<Worker>, Without<Idle>, Without<PathState>),
    >,
    mut dig_query: Query<&mut HitPoints>,
    tilemap_query: Query<&mut TileEntities>,
    mut designations: ResMut<Designations>,
) {
    if query.is_empty() {
        return;
    }

    let map_entities = tilemap_query.single();

    for (entity, pos, job, mut cooldown) in &mut query {
        if !cooldown.0.finished() {
            continue;
        }

        let mut did_work = false;

        match job {
            Job::Dig(dig_pos) => {
                let Some(tile_entity) = map_entities.entities[dig_pos.x][dig_pos.y] else {
                    warn!("Working trying to dig at position without entity.");
                    continue;
                };

                let Ok(mut hp) = dig_query.get_mut(tile_entity) else {
                    warn!("Worker trying dig entity without hitpoints.");
                    continue;
                };

                hp.sub(1);

                if hp.is_zero() {
                    commands.entity(entity).insert(Idle);
                    commands.entity(entity).remove::<Job>();

                    // TODO RemoveDesignationEvent?
                    if let Some(designation) = &designations.0[dig_pos.x][dig_pos.y] {
                        commands.entity(designation.indicator).despawn()
                    }
                    designations.0[dig_pos.x][dig_pos.y] = None;
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
