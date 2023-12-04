use bevy::prelude::*;

use crate::hit_points::HitPoints;

pub struct WorkerPlugin;
impl Plugin for WorkerPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Worker;

#[derive(Bundle)]
pub struct WorkerBundle {
    sheet: SpriteSheetBundle,
    hit_points: HitPoints,
    Worker: Worker,
}
