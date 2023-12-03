use bevy::prelude::*;

pub struct WavesPlugin;
impl Plugin for WavesPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Waves;
