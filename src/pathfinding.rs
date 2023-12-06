use bevy::prelude::*;

use crate::tilemap::{TileKind, TilePos, Tilemap};

pub struct PathfindingPlugin;
impl Plugin for PathfindingPlugin {
    fn build(&self, _app: &mut App) {}
}

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
impl From<Vec<TilePos>> for PathState {
    fn from(path: Vec<TilePos>) -> Self {
        Self { index: 0, path }
    }
}
impl PathState {
    pub fn finished(&self) -> bool {
        self.index + 2 > self.path.len()
    }
}

pub fn enemy_cost_fn(map: &Tilemap) -> impl '_ + Fn(TilePos) -> isize {
    move |pos| {
        let Some(row) = map.tiles.get(pos.x) else {
            return -1;
        };
        let Some(tile) = row.get(pos.y) else {
            return -1;
        };

        match tile {
            TileKind::Dirt => 3,
            TileKind::Road | TileKind::Bridge | TileKind::Spawn => 1,
            _ => -1,
        }
    }
}

pub fn worker_cost_fn(map: &Tilemap) -> impl '_ + Fn(TilePos) -> isize {
    move |pos| {
        let Some(row) = map.tiles.get(pos.x) else {
            return -1;
        };
        let Some(tile) = row.get(pos.y) else {
            return -1;
        };

        // Workers avoid roads, which is where enemies typically are found.

        match tile {
            TileKind::Dirt | TileKind::StoneTunnel => 1,
            TileKind::Road | TileKind::Bridge | TileKind::Home => 3,
            _ => -1,
        }
    }
}

pub fn heuristic(a: TilePos, b: TilePos) -> u32 {
    let absdiff = (IVec2::new(a.x as i32, a.y as i32) - IVec2::new(b.x as i32, b.y as i32)).abs();
    (absdiff.x + absdiff.y) as u32
}

const NEIGHBORS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub struct NeighborCostIter<F> {
    pos: TilePos,
    index: usize,
    cost_fn: F,
}

impl<F> NeighborCostIter<F>
where
    F: Sync + Fn(TilePos) -> isize,
{
    pub fn new(pos: TilePos, cost_fn: F) -> NeighborCostIter<F> {
        NeighborCostIter {
            pos,
            index: 0,
            cost_fn,
        }
    }
}

impl<F> Iterator for NeighborCostIter<F>
where
    F: Fn(TilePos) -> isize,
{
    type Item = (TilePos, u32);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..NEIGHBORS.len() {
            let n = NEIGHBORS.get(i)?;

            let x = (self.pos.x as isize + n.0) as usize;
            let y = (self.pos.y as isize + n.1) as usize;

            let pos = TilePos { x, y };

            let cost = (self.cost_fn)(pos);
            if cost == -1 {
                continue;
            }

            self.index = i + 1;

            return Some((TilePos { x, y }, cost as u32));
        }

        None
    }
}
