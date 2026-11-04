use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    GameState,
    level::{LevelConfig, LevelHandle},
};

pub struct CurrencyPlugin;
impl Plugin for CurrencyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Currency>();
        app.add_systems(OnEnter(GameState::Playing), init);
        app.add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

pub struct NotEnoughCurrencyError;
#[derive(Resource, Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct Currency {
    pub metal: u32,
    pub crystal: u32,
    pub stone: u32,
}
impl Currency {
    pub const ZERO: Self = Self {
        metal: 0,
        crystal: 0,
        stone: 0,
    };

    #[allow(unused)]
    pub fn metal(metal: u32) -> Self {
        Self {
            metal,
            crystal: 0,
            stone: 0,
        }
    }
    #[allow(unused)]
    pub fn crytal(crystal: u32) -> Self {
        Self {
            metal: 0,
            crystal,
            stone: 0,
        }
    }
    #[allow(unused)]
    pub fn stone(stone: u32) -> Self {
        Self {
            metal: 0,
            crystal: 0,
            stone,
        }
    }
    pub fn has(&self, value: &Currency) -> bool {
        self.metal >= value.metal && self.crystal >= value.crystal && self.stone >= value.stone
    }
    pub fn try_sub(&mut self, value: &Currency) -> Result<(), NotEnoughCurrencyError> {
        if self.has(value) {
            self.metal -= value.metal;
            self.crystal -= value.crystal;
            self.stone -= value.stone;
            return Ok(());
        }

        Err(NotEnoughCurrencyError)
    }
    pub fn add(&mut self, value: &Currency) {
        self.metal += value.metal;
        self.crystal += value.crystal;
        self.stone += value.stone;
    }
}

impl Default for Currency {
    fn default() -> Self {
        Self {
            metal: 9999,
            crystal: 9999,
            stone: 9999,
        }
    }
}

fn init(mut commands: Commands, levels: Res<Assets<LevelConfig>>, level_handle: Res<LevelHandle>) {
    let Some(level) = levels.get(&level_handle.0) else {
        warn!("Couldn't find level when initializing Currency ");
        return;
    };
    commands.insert_resource(level.currency.clone());
}

fn cleanup(mut commands: Commands) {
    commands.insert_resource(Currency::default());
}
