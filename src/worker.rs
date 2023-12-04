use crate::{
    hit_points::HitPoints,
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
            .add_systems(OnEnter(GameState::Playing), init);
    }
}

const WORKER_SPRITES: [usize; 2] = [103 * 14 + 0, 103 * 15 + 0];

#[derive(Component, Default)]
pub struct Worker;

#[derive(Bundle, Default)]
pub struct WorkerBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    worker: Worker,
    pos: TilePos,
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
