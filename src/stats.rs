use bevy::prelude::*;

use crate::GameState;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>()
            .add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

#[derive(Resource, Default)]
pub struct Stats {
    pub kills: usize,
    pub mined: usize,
    pub towers: usize,
}

fn cleanup(mut commands: Commands) {
    commands.insert_resource(Stats::default());
}
