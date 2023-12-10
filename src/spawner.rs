use std::time::Duration;

use bevy::{audio::Volume, prelude::*, window::PrimaryWindow};
use bevy_nine_slice_ui::NineSliceTexture;
use serde::Deserialize;

use crate::{
    enemy::{EnemyKind, SpawnEnemyEvent},
    settings::SfxSetting,
    sound::SoundAssets,
    tilemap::{AtlasHandle, TilePos, SCALE, TILE_SIZE},
    ui::UiAssets,
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
            )
            .add_systems(OnExit(GameState::GameOver), cleanup);
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
pub struct SpawnerPortrait;

#[derive(Component)]
pub struct SpawnerContainer;

#[derive(Component)]
pub struct SpawnerIndex(pub usize);

#[derive(Deserialize, Clone)]
pub struct Spawn {
    pub spawner: usize,
    pub num: usize,
    pub delay: f32,
    pub interval: f32,
    pub hp: u32,
    pub kind: EnemyKind,
}

#[derive(Resource, Default)]
pub struct SpawnerStates {
    pub states: Vec<SpawnerState>,
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
        let mut spawn_timer = Timer::from_seconds(spawn.interval, TimerMode::Repeating);
        spawn_timer.set_elapsed(Duration::from_secs_f32(spawn.interval));

        Self {
            delay_timer: Timer::from_seconds(spawn.delay, TimerMode::Once),
            spawn_timer,
            remaining: spawn.num,
            spawn,
        }
    }
}

fn spawn(
    mut commands: Commands,
    mut states: ResMut<SpawnerStates>,
    time: Res<Time>,
    mut waves: ResMut<Waves>,
    mut events: EventWriter<SpawnEnemyEvent>,
    spawners: Query<(&TilePos, &SpawnerIndex)>,
    sound_assets: Res<SoundAssets>,
    sfx_setting: Res<SfxSetting>,
) {
    if states.states.is_empty() {
        return;
    }

    for state in &mut states.states {
        if state.remaining == 0 {
            continue;
        }

        state.delay_timer.tick(time.delta());
        if !state.delay_timer.finished() {
            continue;
        }

        state.spawn_timer.tick(time.delta());
        if state.spawn_timer.just_finished() {
            let Some((pos, _)) = spawners.iter().find(|(_, i)| i.0 == state.spawn.spawner) else {
                warn!("Couldn't fetch position of spawner.");
                continue;
            };

            events.send(SpawnEnemyEvent {
                pos: *pos,
                kind: state.spawn.kind,
                hp: state.spawn.hp,
            });

            state.remaining -= 1;
        }
    }

    let none_remaining = states.states.iter().all(|s| s.remaining == 0);
    if none_remaining {
        info!("Wave {}: All spawners have finished.", waves.current);
        let next = waves.advance();

        if next.is_some() {
            commands.spawn(AudioBundle {
                source: sound_assets.wave.clone(),
                settings: PlaybackSettings::DESPAWN
                    .with_volume(Volume::new_absolute(**sfx_setting as f32 / 100.)),
            });
        }
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
    query: Query<Entity, Added<Spawner>>,
    ui_assets: Res<UiAssets>,
    atlas_handle: Res<AtlasHandle>,
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
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                NineSliceTexture::from_image(ui_assets.nine_slice.clone()),
                SpawnerContainer,
            ))
            .with_children(|parent| {
                parent.spawn((
                    AtlasImageBundle {
                        style: Style {
                            width: Val::Px(TILE_SIZE.x * SCALE.x),
                            height: Val::Px(TILE_SIZE.y * SCALE.y),
                            ..default()
                        },
                        texture_atlas: atlas_handle.0.clone(),
                        texture_atlas_image: UiTextureAtlasImage {
                            index: 103 * 8,
                            ..default()
                        },
                        ..default()
                    },
                    SpawnerPortrait,
                ));
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
                        margin: UiRect::top(Val::Px(4.)),
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
    query: Query<(&Transform, &SpawnerIndex, &SpawnerUi)>,
    mut ui_query: Query<(&mut Style, &Children), With<SpawnerContainer>>,
    mut ui_image_query: Query<&mut UiTextureAtlasImage, With<SpawnerPortrait>>,
    mut ui_text_query: Query<&mut Text, With<SpawnerDelayText>>,
    spawners: Res<SpawnerStates>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    for (_, index, ui_entity) in &query {
        let Ok((mut container_style, children)) = ui_query.get_mut(ui_entity.0) else {
            continue;
        };

        let mut text_iter = ui_text_query.iter_many_mut(children);
        let Some(mut text) = text_iter.fetch_next() else {
            continue;
        };

        let Some(state) = spawners.states.get(index.0) else {
            continue;
        };

        if state.remaining > 0 && !state.delay_timer.finished() {
            container_style.display = Display::Flex;
        } else {
            container_style.display = Display::None;
            continue;
        }

        let mut image_iter = ui_image_query.iter_many_mut(children);
        let Some(mut image) = image_iter.fetch_next() else {
            continue;
        };

        image.index = state.spawn.kind.atlas_index();

        text.sections[0].value = format!("{:.1}", state.delay_timer.remaining_secs());
    }

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    for (transform, _, ui_entity) in &query {
        let inset = 10.;
        let indicator_rect =
            Vec2::new(window.width(), window.height()) / 2. - SPAWNER_UI_SIZE / 2. - inset / 2.;

        let camera_pos = camera_transform.translation().truncate();

        let clamped = transform
            .translation
            .truncate()
            .clamp(camera_pos - indicator_rect, camera_pos + indicator_rect);

        let Ok((mut container_style, _)) = ui_query.get_mut(ui_entity.0) else {
            continue;
        };

        let Some(viewport) = camera.world_to_viewport(camera_transform, clamped.extend(0.)) else {
            continue;
        };

        container_style.left = Val::Px(viewport.x - SPAWNER_UI_SIZE.x / 2.);
        container_style.top = Val::Px(viewport.y - SPAWNER_UI_SIZE.y / 2.);
    }
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Spawner>, With<SpawnerContainer>)>>,
    mut waves: ResMut<Waves>,
    mut states: ResMut<SpawnerStates>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }

    states.states.clear();

    waves.reset();

    if let Some(wave) = waves.current() {
        for spawn in wave.spawns.iter().cloned() {
            states.states.push(spawn.into());
        }
    }
}
