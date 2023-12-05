use std::ops::SubAssign;

use bevy::prelude::*;

pub struct HitPointsPlugin;
impl Plugin for HitPointsPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Debug)]
pub struct HitPoints {
    current: u32,
    max: u32,
}
impl Default for HitPoints {
    fn default() -> Self {
        Self::full(1)
    }
}
impl HitPoints {
    pub fn full(val: u32) -> Self {
        Self {
            current: val,
            max: val,
        }
    }
    pub fn sub(&mut self, val: u32) {
        self.current = self.current.saturating_sub(val);
    }
    pub fn is_zero(&self) -> bool {
        self.current == 0
    }
}
