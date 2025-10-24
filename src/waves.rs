use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    GameState,
    level::{LevelConfig, LevelHandle},
    spawner::Spawn,
};

pub struct WavesPlugin;
impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init);
    }
}

#[derive(Resource)]
pub struct Waves {
    pub current: usize,
    pub waves: Vec<Wave>,
}
impl Waves {
    pub fn current(&self) -> Option<&Wave> {
        self.waves.get(self.current)
    }
    pub fn advance(&mut self) -> Option<&Wave> {
        self.current += 1;
        self.current()
    }
    pub fn reset(&mut self) {
        self.current = 0;
    }
}
impl From<Vec<Wave>> for Waves {
    fn from(waves: Vec<Wave>) -> Self {
        Self { current: 0, waves }
    }
}

#[derive(Deserialize, Clone)]
pub struct Wave {
    pub spawns: Vec<Spawn>,
}

#[derive(Message)]
pub struct WaveStartMessage;

pub fn init(
    mut commands: Commands,
    level_handle: Res<LevelHandle>,
    levels: Res<Assets<LevelConfig>>,
) {
    if let Some(level) = levels.get(&level_handle.0) {
        commands.insert_resource::<Waves>(level.waves.clone().into());
    }
}
