use bevy::prelude::*;

pub struct HomePlugin;
impl Plugin for HomePlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
pub struct Home;
