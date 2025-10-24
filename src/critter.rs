use std::time::Duration;

use bevy::prelude::*;
use rand::{rngs::SmallRng, seq::IndexedRandom, Rng, SeedableRng};
use serde::Deserialize;

use crate::{
    level::{LevelConfig, LevelHandle},
    main_menu::MainMenuAssets,
    movement::{MovingProgress, Speed},
    pathfinding::{critter_cost_fn, heuristic, NeighborCostIter, PathState, SquareAreaCostIter},
    tilemap::{AtlasHandle, Map, TilePos},
    util::cleanup,
    GameState,
};
use pathfinding::prelude::astar;

pub struct CritterPlugin;
impl Plugin for CritterPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnCritterMessage>()
            .init_resource::<CritterRng>()
            .add_systems(OnEnter(GameState::Playing), setup)
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (spawn, pathfinding, behavior, idle)
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::MainMenu))),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup::<CritterKind>)
            .add_systems(OnExit(GameState::MainMenu), cleanup::<CritterKind>);
    }
}

#[derive(Component, Default, Deserialize, Debug, Copy, Clone)]
#[require(Sprite, TilePos, MovingProgress, Speed, IdleTimer, CritterBehavior)]
pub enum CritterKind {
    #[default]
    Llama,
    Snake,
    Whale,
}
impl CritterKind {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Llama => 103 * 18 + 5,
            Self::Snake => 103 * 18 + 2,
            Self::Whale => 103 * 18 + 10,
        }
    }
}

#[derive(Message)]
pub struct SpawnCritterMessage {
    pub kind: CritterKind,
    pub pos: TilePos,
}

#[derive(Component, Default)]
enum CritterBehavior {
    #[default]
    Idle,
    RandomWalk,
}

#[derive(Component)]
pub struct IdleTimer(Timer);
impl Default for IdleTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(10., TimerMode::Once))
    }
}

#[derive(Resource)]
pub struct CritterRng(SmallRng);
impl Default for CritterRng {
    fn default() -> Self {
        Self(SmallRng::from_os_rng())
    }
}

fn setup(
    mut messages: MessageWriter<SpawnCritterMessage>,
    level_handle: Res<LevelHandle>,
    levels: Res<Assets<LevelConfig>>,
) {
    let Some(level) = levels.get(&level_handle.0) else {
        warn!("Couldn't find level when spawning critters.");
        return;
    };

    for (pos, kind) in &level.critters {
        messages.write(SpawnCritterMessage {
            kind: *kind,
            pos: *pos,
        });
    }
}

fn setup_main_menu(
    mut messages: MessageWriter<SpawnCritterMessage>,
    main_menu_assets: Res<MainMenuAssets>,
    levels: Res<Assets<LevelConfig>>,
) {
    let Some(level) = levels.get(&main_menu_assets.level) else {
        warn!("Couldn't find level when spawning critters.");
        return;
    };

    for (pos, kind) in &level.critters {
        messages.write(SpawnCritterMessage {
            kind: *kind,
            pos: *pos,
        });
    }
}

// TODO consider replacing with OnAdd observer
fn spawn(
    mut commands: Commands,
    mut messages: MessageReader<SpawnCritterMessage>,
    atlas_handle: Res<AtlasHandle>,
    tilemap_query: Query<&Map>,
    mut rng: ResMut<CritterRng>,
) {
    for message in messages.read() {
        let Ok(tilemap) = tilemap_query.single() else {
            continue;
        };

        let world = tilemap.pos_to_world(message.pos);

        commands.spawn((
            Sprite {
                image: atlas_handle.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_handle.layout.clone(),
                    index: message.kind.atlas_index(),
                }),
                ..default()
            },
            Transform {
                translation: world.extend(1.),
                scale: crate::tilemap::SCALE.extend(1.),
                ..default()
            },
            IdleTimer(Timer::new(
                Duration::from_secs_f32(rng.0.random_range(4.0..14.0)),
                TimerMode::Once,
            )),
            message.kind,
            message.pos,
            Speed(1.),
            Name::new("Critter"),
        ));
    }
}

fn pathfinding(
    mut commands: Commands,
    query: Query<(Entity, &TilePos, &CritterBehavior, &CritterKind), Without<PathState>>,
    tilemap_query: Query<&Map>,
    mut rng: ResMut<CritterRng>,
) {
    for (entity, pos, behavior, kind) in &query {
        if !matches!(behavior, CritterBehavior::RandomWalk) {
            continue;
        }

        let Ok(map) = tilemap_query.single() else {
            return;
        };

        // Choose a random neighboring tile
        let neighbors =
            SquareAreaCostIter::new(*pos, 2, critter_cost_fn(map, *kind)).collect::<Vec<_>>();
        let Some((goal, _)) = neighbors.choose(&mut rng.0) else {
            warn!("{:?} is stuck.", kind);
            continue;
        };

        let Some(result) = astar(
            pos,
            |p| NeighborCostIter::new(*p, critter_cost_fn(map, *kind)),
            |p| heuristic(*p, *goal),
            |p| *p == *goal,
        ) else {
            warn!("{:?} is unable to find path to goal.", kind);
            continue;
        };

        // The critter may have been despawned in the same frame.
        commands
            .entity(entity)
            .try_insert(PathState::from(result.0));

        // limit the amount of pathfinding we do each frame.
        break;
    }
}

fn behavior(
    mut removed: RemovedComponents<PathState>,
    mut query: Query<(&mut CritterBehavior, &mut IdleTimer), Without<PathState>>,
) {
    // We've reached our destination whenever a `PathState` has been removed.

    let mut critter_iter = query.iter_many_mut(removed.read());
    while let Some((mut behavior, mut timer)) = critter_iter.fetch_next() {
        if matches!(*behavior, CritterBehavior::RandomWalk) {
            *behavior = CritterBehavior::Idle;
            timer.0.reset();
        }
    }
}

fn idle(mut critters: Query<(&mut CritterBehavior, &mut IdleTimer)>, time: Res<Time>) {
    for (mut behavior, mut timer) in critters
        .iter_mut()
        .filter(|(behavior, _)| matches!(**behavior, CritterBehavior::Idle))
    {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            *behavior = CritterBehavior::RandomWalk;
        }
    }
}
