use bevy::prelude::*;

pub struct HomePlugin;
impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Home;
