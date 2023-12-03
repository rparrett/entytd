use bevy::prelude::*;

use crate::hit_points::HitPoints;

pub struct EnemyPlugin;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Enemy;

#[derive(Bundle)]
pub struct EnemyBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    enemy: Enemy,
}
