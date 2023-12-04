use std::time::Duration;

use bevy::{prelude::*, utils::FloatOrd, window::PrimaryWindow};
use bevy_nine_slice_ui::NineSliceTexture;
use serde::Deserialize;

use crate::{
    waves::{WaveStartEvent, Waves},
    GameState,
};

const SPAWNER_UI_SIZE: Vec2 = Vec2::new(60., 60.);

pub struct SpawnerPlugin;
impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WaveStartEvent>()
            .init_resource::<SpawnerStates>()
            .add_systems(Update, (init, spawn).run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                (add_spawner_ui, update_spawner_ui).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Spawner;

/// Points to the UI element that shows the spawner's state.
#[derive(Component)]
pub struct SpawnerUi(Entity);

#[derive(Component)]
pub struct SpawnerDelayText;

#[derive(Component)]
pub struct SpawnerContainer;

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
            info!("Spawning!");

            state.remaining -= 1;
        }
    }

    let none_remaining = spawners.states.iter().all(|s| s.remaining == 0);
    if none_remaining {
        info!("All spawners have finished.");
        let _ = waves.advance();
    }
}

fn init(waves: Res<Waves>, mut states: ResMut<SpawnerStates>) {
    if !waves.is_changed() {
        return;
    }

    states.states.clear();

    if let Some(wave) = waves.current() {
        for spawn in wave.spawns.iter().cloned() {
            states.states.push(spawn.into());
        }
    }
}

fn add_spawner_ui(
    mut commands: Commands,
    server: Res<AssetServer>,
    query: Query<Entity, Added<Spawner>>,
) {
    for entity in &query {
        let ui_entity = commands
            .spawn((
                NodeBundle {
                    style: Style {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        width: Val::Px(SPAWNER_UI_SIZE.x),
                        height: Val::Px(SPAWNER_UI_SIZE.y),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                    ..default()
                },
                NineSliceTexture::from_image(server.load("ui_nine_slice.png")),
                SpawnerContainer,
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "10.1",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    )
                    .with_style(Style {
                        margin: UiRect::bottom(Val::Px(8.)),
                        ..default()
                    }),
                    SpawnerDelayText,
                ));
            })
            .id();

        commands.entity(entity).insert(SpawnerUi(ui_entity));
    }
}

fn update_spawner_ui(
    query: Query<(&Transform, &SpawnerUi)>,
    mut ui_query: Query<(&mut Style, &Children), With<SpawnerContainer>>,
    mut ui_text_query: Query<&mut Text, With<SpawnerDelayText>>,
    spawners: Res<SpawnerStates>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    // spawners are indexed top-down, left-right
    // TODO sort on level spawn and store index instead?
    let mut uis = query.iter().collect::<Vec<_>>();
    uis.sort_by_key(|(t, _)| (FloatOrd(-t.translation.y), FloatOrd(t.translation.x)));

    for (index, (_, ui_entity)) in uis.iter().enumerate() {
        let Ok((mut container_style, children)) = ui_query.get_mut(ui_entity.0) else {
            continue;
        };

        let mut text_iter = ui_text_query.iter_many_mut(children);
        let Some(mut text) = text_iter.fetch_next() else {
            continue;
        };

        let Some(state) = spawners.states.get(index) else {
            continue;
        };

        if state.remaining > 0 && !state.delay_timer.finished() {
            container_style.display = Display::Flex;
        } else {
            container_style.display = Display::None;
            continue;
        }

        text.sections[0].value = format!("{:.1}", state.delay_timer.remaining_secs());
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (transform, ui_entity) in &mut uis {
        let diff = transform.translation.truncate() - camera_transform.translation().truncate();

        let inset = 12.;
        let indicator_rect =
            Vec2::new(window.width(), window.height()) / 2. - SPAWNER_UI_SIZE / 2. - inset / 2.;

        let projection =
            crate::util::project_onto_bounding_rectangle(diff, -indicator_rect, indicator_rect)
                .unwrap();

        let Ok((mut container_style, _)) = ui_query.get_mut(ui_entity.0) else {
            continue;
        };

        let world = camera_transform.translation().truncate() + projection.0;
        let Some(viewport) = camera.world_to_viewport(camera_transform, world.extend(0.)) else {
            continue;
        };

        container_style.left = Val::Px(viewport.x - SPAWNER_UI_SIZE.x / 2.);
        container_style.top = Val::Px(viewport.y - SPAWNER_UI_SIZE.y / 2.);
    }
}
