use bevy::prelude::*;

use crate::GameState;

pub struct CurrencyPlugin;
impl Plugin for CurrencyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Currency>();
        app.add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

pub struct NotEnoughCurrencyError;
#[derive(Resource, Debug, Default, Eq, PartialEq)]
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
}

fn cleanup(mut commands: Commands) {
    commands.insert_resource(Currency::default());
}
