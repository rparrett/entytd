use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct HitPoints {
    pub current: u32,
    pub max: u32,
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
    pub fn fraction(&self) -> f32 {
        self.current as f32 / self.max as f32
    }
}
