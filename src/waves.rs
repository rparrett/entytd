use bevy::prelude::*;
use serde::Deserialize;

use crate::GameState;

pub struct WavesPlugin;
impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), queue_load);
    }
}

#[derive(Deserialize)]
pub struct Waves(pub Vec<Wave>);

#[derive(Deserialize)]
pub struct Wave {
    delay: f32,
    spawns: Vec<Spawn>,
}

#[derive(Deserialize)]
pub struct Spawn {
    spawner: usize,
    num: usize,
    interval: f32,
    hp: u32,
    // TODO element
}

fn queue_load(asset_server: Res<AssetServer>) {}
