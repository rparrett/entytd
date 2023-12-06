use std::time::Duration;

use bevy::{
    core::FrameCount,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    common_assets::CommonAssets,
    hit_points::HitPoints,
    home::Home,
    tilemap::{AtlasHandle, SCALE, TILE_SIZE},
    worker::{Idle, Worker},
    GameState,
};

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityCountUpdateTimer>()
            .init_resource::<FpsUpdateTimer>()
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(
                Update,
                (
                    update_entity_count,
                    update_idle_workers,
                    update_fps,
                    update_home_hit_points,
                ),
            );
    }
}

#[derive(Component)]
pub struct HudContainer;

#[derive(Component, Default)]
pub struct EntityCount;

#[derive(Component, Default)]
pub struct Fps;

#[derive(Component, Default)]
pub struct IdleWorkers;

#[derive(Component, Default)]
pub struct Crystal;

#[derive(Component, Default)]
pub struct Metal;

#[derive(Component, Default)]
pub struct HomeHitPoints;

#[derive(Resource)]
pub struct EntityCountUpdateTimer(Timer);
impl Default for EntityCountUpdateTimer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(1., TimerMode::Repeating);
        timer.set_elapsed(Duration::from_secs_f32(1.0 - f32::EPSILON));
        Self(timer)
    }
}

#[derive(Resource)]
pub struct FpsUpdateTimer(Timer);
impl Default for FpsUpdateTimer {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.2, TimerMode::Repeating);
        timer.set_elapsed(Duration::from_secs_f32(1.0 - f32::EPSILON));
        Self(timer)
    }
}

fn init(mut commands: Commands, common: Res<CommonAssets>, atlas_handle: Res<AtlasHandle>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    height: Val::Percent(100.),
                    width: Val::Px(100.),
                    left: Val::Px(5.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            Name::new("HudContainer"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            position_type: PositionType::Absolute,
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexStart,
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    NineSliceTexture::from_image(common.ui_nine_slice.clone()),
                    HudContainer,
                ))
                .with_children(|parent| {
                    init_hud_item::<EntityCount>(parent, atlas_handle.0.clone(), 103 * 47 + 101);
                    init_hud_item::<Fps>(parent, atlas_handle.0.clone(), 103 * 49 + 78);
                    init_hud_item::<IdleWorkers>(parent, atlas_handle.0.clone(), 103 * 15 + 24);
                    init_hud_item::<Crystal>(parent, atlas_handle.0.clone(), 103 * 24 + 0);
                    init_hud_item::<Metal>(parent, atlas_handle.0.clone(), 103 * 25 + 6);
                    init_hud_item::<HomeHitPoints>(parent, atlas_handle.0.clone(), 103 * 33 + 24);
                });
        });
}

fn init_hud_item<M: Component + Default>(
    commands: &mut ChildBuilder,
    texture_atlas: Handle<TextureAtlas>,
    atlas_index: usize,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                ..default()
            },
            Name::new("HudItem"),
            M::default(),
        ))
        .with_children(|parent| {
            parent.spawn(AtlasImageBundle {
                style: Style {
                    width: Val::Px(TILE_SIZE.x * SCALE.x),
                    height: Val::Px(TILE_SIZE.y * SCALE.y),
                    margin: UiRect::right(Val::Px(5.)),
                    ..default()
                },
                texture_atlas,
                texture_atlas_image: UiTextureAtlasImage {
                    index: atlas_index,
                    ..default()
                },
                ..default()
            });
            parent.spawn(TextBundle::from_section(
                "10.1",
                TextStyle {
                    font_size: 18.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
}

fn update_entity_count(
    entities: Query<Entity>,
    time: Res<Time>,
    mut timer: ResMut<EntityCountUpdateTimer>,
    item_query: Query<&Children, With<EntityCount>>,
    mut text_query: Query<&mut Text>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    text.sections[0].value = format!("{}", entities.iter().len());
}

fn update_idle_workers(
    has_idle: Query<(), (With<Worker>, With<Idle>)>,
    hasnt_idle: Query<(), (With<Worker>, Without<Idle>)>,
    added_idle: Query<(), Added<Idle>>,
    removed_idle: RemovedComponents<Idle>,
    item_query: Query<&Children, With<IdleWorkers>>,
    mut text_query: Query<&mut Text>,
) {
    if added_idle.is_empty() && removed_idle.is_empty() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    let idle = has_idle.iter().count();
    let not_idle = hasnt_idle.iter().count();

    text.sections[0].value = format!("{}/{}", idle, idle + not_idle);
}

fn update_fps(
    time: Res<Time>,
    mut timer: ResMut<FpsUpdateTimer>,
    diagnostics: Res<DiagnosticsStore>,
    item_query: Query<&Children, With<Fps>>,
    mut text_query: Query<&mut Text>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    text.sections[0].value = format!("{fps:.1}");
}

fn update_home_hit_points(
    query: Query<&HitPoints, With<Home>>,
    changed_query: Query<(), (With<Home>, Changed<HitPoints>)>,
    item_query: Query<&Children, With<HomeHitPoints>>,
    mut text_query: Query<&mut Text>,
) {
    if changed_query.is_empty() {
        return;
    }

    let (current, max) = query
        .iter()
        .fold((0, 0), |sum, hp| (sum.0 + hp.current, sum.1 + hp.max));

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    // TODO color

    text.sections[0].value = format!("{current}/{max}");
}
