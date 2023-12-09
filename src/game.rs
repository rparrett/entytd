use bevy::prelude::*;

use crate::{
    enemy::Enemy, hit_points::HitPoints, home::Home, spawner::SpawnerStates, waves::Waves,
    GameState,
};

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Won>().add_systems(
            Update,
            (check_win, check_loss).run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct Won(pub bool);

fn check_win(
    waves: Res<Waves>,
    spawners: Res<SpawnerStates>,
    enemies: Query<(), With<Enemy>>,
    mut won: ResMut<Won>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if waves.current().is_some() {
        return;
    }

    if spawners.states.iter().any(|s| s.remaining > 0) {
        return;
    }

    if enemies.iter().len() > 0 {
        return;
    }

    won.0 = true;

    next_state.set(GameState::GameOver);
}

fn check_loss(
    changed: Query<(), (With<Home>, Changed<HitPoints>)>,
    query: Query<&HitPoints, With<Home>>,
    mut won: ResMut<Won>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if changed.iter().count() == 0 {
        return;
    }

    if query.iter().any(|hp| !hp.is_zero()) {
        return;
    }

    won.0 = false;

    next_state.set(GameState::GameOver);
}
