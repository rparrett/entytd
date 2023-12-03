use bevy::prelude::*;

pub struct SpawnerPlugin;
impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Spawner;
