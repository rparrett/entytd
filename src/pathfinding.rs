use bevy::prelude::*;

use crate::{
    critter::CritterKind,
    enemy::EnemyKind,
    tilemap::{Map, TileKind, TilePos},
};

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
        self.index + 1 > self.path.len() - 1
    }
}

pub fn enemy_cost_fn(map: &Map, kind: EnemyKind) -> impl '_ + Fn((isize, isize)) -> isize {
    move |pos| {
        let Some(tile) = map.0.get(pos.1, pos.0) else {
            return -1;
        };

        match (tile, kind) {
            (TileKind::Dirt, _) => 5,
            (
                TileKind::Forest,
                EnemyKind::Ent | EnemyKind::EntTwo | EnemyKind::EntThree | EnemyKind::EntFour,
            )
            | (
                TileKind::Road
                | TileKind::Bridge
                | TileKind::Spawn
                | TileKind::DirtPathNSA
                | TileKind::DirtPathNSB
                | TileKind::DirtPathEWA
                | TileKind::DirtPathEWB
                | TileKind::DirtPathSW
                | TileKind::DirtPathNW
                | TileKind::DirtPathSE
                | TileKind::DirtPathNE
                | TileKind::DirtPathNSW
                | TileKind::DirtPathSEW
                | TileKind::DirtPathNSE
                | TileKind::DirtPathNEW
                | TileKind::DirtPathNSEW,
                _,
            ) => 1,
            _ => -1,
        }
    }
}

pub fn worker_cost_fn(map: &Map) -> impl '_ + Fn((isize, isize)) -> isize {
    move |pos| {
        let Some(tile) = map.0.get(pos.1, pos.0) else {
            return -1;
        };

        // Workers avoid roads, which is where enemies typically are found.

        match tile {
            TileKind::Dirt
            | TileKind::StoneTunnel
            | TileKind::DirtPathNSA
            | TileKind::DirtPathNSB
            | TileKind::DirtPathEWA
            | TileKind::DirtPathEWB
            | TileKind::DirtPathSW
            | TileKind::DirtPathNW
            | TileKind::DirtPathSE
            | TileKind::DirtPathNE
            | TileKind::DirtPathNSW
            | TileKind::DirtPathSEW
            | TileKind::DirtPathNSE
            | TileKind::DirtPathNEW
            | TileKind::DirtPathNSEW => 1,
            TileKind::Road
            | TileKind::Bridge
            | TileKind::Home
            | TileKind::HomeTwo
            | TileKind::HomeDead
            | TileKind::Tower => 5,
            _ => -1,
        }
    }
}

pub fn critter_cost_fn(map: &Map, kind: CritterKind) -> impl '_ + Fn((isize, isize)) -> isize {
    move |pos| {
        let Some(tile) = map.0.get(pos.1, pos.0) else {
            return -1;
        };

        match (tile, kind) {
            (TileKind::Forest | TileKind::GrassA | TileKind::GrassB, CritterKind::Snake) => 1,
            (TileKind::River, CritterKind::Whale) => 1,
            (TileKind::GrassA | TileKind::GrassB, CritterKind::Llama) => 1,
            _ => -1,
        }
    }
}

pub fn heuristic(a: TilePos, b: TilePos) -> u32 {
    let absdiff = (IVec2::new(a.x as i32, a.y as i32) - IVec2::new(b.x as i32, b.y as i32)).abs();
    (absdiff.x + absdiff.y) as u32
}

pub const NEIGHBORS: [(isize, isize); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub struct NeighborCostIter<F> {
    pos: TilePos,
    index: usize,
    cost_fn: F,
}

impl<F> NeighborCostIter<F>
where
    F: Sync + Fn((isize, isize)) -> isize,
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
    F: Fn((isize, isize)) -> isize,
{
    type Item = (TilePos, u32);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.index..NEIGHBORS.len() {
            let offset = NEIGHBORS.get(i)?;

            let neighbor = *offset + self.pos;

            let cost = (self.cost_fn)(neighbor);
            if cost == -1 {
                continue;
            }

            self.index = i + 1;

            return Some((TilePos::from(neighbor), cost as u32));
        }

        None
    }
}
