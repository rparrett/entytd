use bevy::prelude::*;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Enemy;
