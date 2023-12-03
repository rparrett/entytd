use bevy::prelude::*;
use serde::Deserialize;

use crate::{
    waves::{WaveStartEvent, Waves},
    GameState,
};

pub struct SpawnerPlugin;
impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WaveStartEvent>()
            .init_resource::<SpawnerStates>()
            .add_systems(Update, (init, spawn).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Spawner;

#[derive(Deserialize, Clone)]
pub struct Spawn {
    spawner: usize,
    num: usize,
    delay: f32,
    interval: f32,
    hp: u32,
    // TODO enemy type
}

#[derive(Resource, Default)]
pub struct SpawnerStates {
    states: Vec<SpawnerState>,
}
impl From<&Vec<Spawn>> for SpawnerStates {
    fn from(value: &Vec<Spawn>) -> Self {
        let mut states = Self::default();

        states
            .states
            .extend(value.iter().cloned().map(|w| w.into()));

        states
    }
}

pub struct SpawnerState {
    pub delay_timer: Timer,
    pub spawn_timer: Timer,
    pub remaining: usize,
    pub spawn: Spawn,
}
impl From<Spawn> for SpawnerState {
    fn from(spawn: Spawn) -> Self {
        Self {
            delay_timer: Timer::from_seconds(spawn.delay, TimerMode::Once),
            spawn_timer: Timer::from_seconds(spawn.interval, TimerMode::Repeating),
            remaining: spawn.num,
            spawn,
        }
    }
}

fn spawn(mut spawners: ResMut<SpawnerStates>, time: Res<Time>, mut waves: ResMut<Waves>) {
    if spawners.states.len() == 0 {
        return;
    }

    for state in &mut spawners.states {
        if state.remaining == 0 {
            continue;
        }

        state.delay_timer.tick(time.delta());
        if !state.delay_timer.finished() {
            continue;
        }

        state.spawn_timer.tick(time.delta());
        if state.spawn_timer.just_finished() {
            // spawn
            info!("spawning something!");

            state.remaining -= 1;
        }
    }

    let none_remaining = spawners.states.iter().all(|s| s.remaining == 0);
    if none_remaining {
        info!("all spawners finished.");
        let _ = waves.advance();
    }
}

fn init(waves: Res<Waves>, mut commands: Commands) {
    if !waves.is_changed() {
        return;
    }

    info!("wave changed, initializing spawners");

    if let Some(wave) = waves.current() {
        commands.insert_resource::<SpawnerStates>((&wave.spawns).into());
    } else {
        commands.insert_resource(SpawnerStates::default());
    }
}
