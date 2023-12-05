use crate::{
    designate_tool::Designations,
    hit_points::HitPoints,
    movement::{MovingProgress, Speed},
    pathfinding::{PathState, WorkerPathfinding},
    tilemap::{AtlasHandle, TilePos, Tilemap, TilemapHandle},
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
            .add_systems(Update, find_job.run_if(in_state(GameState::Playing)));
    }
}

const WORKER_SPRITES: [usize; 2] = [103 * 14 + 0, 103 * 15 + 0];

#[derive(Component, Default)]
pub struct Worker;

#[derive(Component)]
pub struct Idle;

#[derive(Bundle, Default)]
pub struct WorkerBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    worker: Worker,
    pos: TilePos,
    moving_animation: MovingProgress,
    speed: Speed,
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
        let Some((_goal, path)) = pathfinding.0.find_closest_goal(
            (pos.x, pos.y),
            &neighbors,
            crate::pathfinding::worker_cost_fn(&map),
        ) else {
            warn!("Worker unable to find path to goal.");
            continue;
        };

        let resolved = path.resolve(crate::pathfinding::worker_cost_fn(&map));

        commands
            .entity(entity)
            .insert(PathState::from(resolved))
            .remove::<Idle>();
    }
}
