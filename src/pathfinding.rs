use bevy::prelude::*;
use hierarchical_pathfinding::prelude::*;

use crate::{
    tilemap::{TileKind, TilePos, Tilemap, TilemapHandle},
    GameState,
};

pub struct PathfindingPlugin;
impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, init.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct Pathfinding(pub PathCache<ManhattanNeighborhood>);

#[derive(Component)]
pub struct PathState {
    pub path: Vec<TilePos>,
    pub index: usize,
}
impl From<Vec<(usize, usize)>> for PathState {
    fn from(mut value: Vec<(usize, usize)>) -> Self {
        Self {
            index: 0,
            path: value.drain(..).map(|i| i.into()).collect::<Vec<_>>(),
        }
    }
}
impl PathState {
    pub fn finished(&self) -> bool {
        self.index > self.path.len() - 2
    }
}

pub fn cost_fn(map: &Tilemap) -> impl '_ + Sync + Fn((usize, usize)) -> isize {
    move |(x, y)| match map.tiles[x][y] {
        TileKind::Dirt => 1,
        TileKind::Road | TileKind::Bridge | TileKind::Spawn => 2,
        _ => -1,
    }
}

fn init(
    mut commands: Commands,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
    pathfinding: Option<Res<Pathfinding>>,
) {
    // TODO can this be an onenter system? is the tilemap ready in time?
    if pathfinding.is_some() {
        return;
    }

    let Some(map) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    info!("building pathcache");

    let pathfinding = PathCache::new(
        (map.width, map.height),
        cost_fn(&map),
        ManhattanNeighborhood::new(map.width, map.height),
        PathCacheConfig::with_chunk_size(3),
    );

    commands.insert_resource(Pathfinding(pathfinding));
}
