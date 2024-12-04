use std::time::Duration;

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{
    currency::Currency,
    designate_tool::DesignationKind,
    hit_points::HitPoints,
    home::Home,
    tilemap::{AtlasHandle, SCALE, TILE_SIZE},
    tool_selector::SelectedTool,
    ui::{self, slice_image_mode, UiAssets, TITLE_TEXT},
    waves::Waves,
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
                    update_stone,
                    update_metal,
                    update_crystal,
                    update_wave_count,
                ),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct HudContainer;

#[derive(Component, Default)]
pub struct EntityCount;

#[derive(Component, Default)]
pub struct Fps;

#[derive(Component, Default)]
pub struct IdleWorkers;

#[derive(Component, Default)]
pub struct Stone;

#[derive(Component, Default)]
pub struct Crystal;

#[derive(Component, Default)]
pub struct Metal;

#[derive(Component, Default)]
pub struct HomeHitPoints;

#[derive(Component, Default)]
pub struct WaveCount;

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

fn init(mut commands: Commands, assets: Res<UiAssets>, atlas_handle: Res<AtlasHandle>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.),
                width: Val::Px(100.),
                left: Val::Px(5.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(5.),
                ..default()
            },
            HudRoot,
            Name::new("HudRoot"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(6.)),
                        ..default()
                    },
                    ImageNode {
                        image: assets.nine_panel.clone(),
                        image_mode: slice_image_mode(),
                        ..default()
                    },
                    HudContainer,
                ))
                .with_children(|parent| {
                    init_hud_item::<EntityCount>(
                        parent,
                        "0".to_string(),
                        &atlas_handle,
                        103 * 47 + 101,
                    );
                    init_hud_item::<Fps>(parent, "0".to_string(), &atlas_handle, 103 * 49 + 78);
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(6.)),
                        ..default()
                    },
                    ImageNode {
                        image: assets.nine_panel.clone(),
                        ..default()
                    },
                    // 9 slice
                    HudContainer,
                ))
                .with_children(|parent| {
                    init_hud_item::<HomeHitPoints>(
                        parent,
                        "0/0".to_string(),
                        &atlas_handle,
                        103 * 33 + 24,
                    );
                    init_hud_item::<IdleWorkers>(
                        parent,
                        "0/0".to_string(),
                        &atlas_handle,
                        103 * 15 + 24,
                    );
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(6.)),
                        ..default()
                    },
                    ImageNode {
                        image: assets.nine_panel.clone(),
                        image_mode: slice_image_mode(),
                        ..default()
                    },
                    HudContainer,
                ))
                .with_children(|parent| {
                    init_hud_item::<Stone>(parent, "0".to_string(), &atlas_handle, 103 * 2 + 5);
                    init_hud_item::<Metal>(parent, "0".to_string(), &atlas_handle, 103 * 25 + 6);
                    init_hud_item::<Crystal>(parent, "0".to_string(), &atlas_handle, 103 * 24);
                });

            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::FlexStart,
                        padding: UiRect::all(Val::Px(6.)),
                        ..default()
                    },
                    ImageNode {
                        image: assets.nine_panel.clone(),
                        image_mode: slice_image_mode(),
                        ..default()
                    },
                    HudContainer,
                ))
                .with_children(|parent| {
                    init_hud_item::<WaveCount>(
                        parent,
                        "0/0".to_string(),
                        &atlas_handle,
                        103 * 48 + 94,
                    );
                });
        });
}

fn init_hud_item<M: Component + Default>(
    commands: &mut ChildBuilder,
    text: String,
    atlas_handle: &AtlasHandle,
    atlas_index: usize,
) {
    commands
        .spawn((
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(5.)),
                ..default()
            },
            Name::new("HudItem"),
            M::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Px(TILE_SIZE.x * SCALE.x),
                    height: Val::Px(TILE_SIZE.y * SCALE.y),
                    margin: UiRect::right(Val::Px(5.)),
                    ..default()
                },
                ImageNode {
                    image: atlas_handle.image.clone().into(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_handle.layout.clone(),
                        index: atlas_index,
                    }),
                    ..default()
                },
            ));
            parent
                .spawn((
                    Text::new(text),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(TITLE_TEXT),
                ))
                .with_child((
                    Text::default(),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(TITLE_TEXT),
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

    text.0 = format!("{}", entities.iter().len());
}

fn update_idle_workers(
    has_idle: Query<(), (With<Worker>, With<Idle>)>,
    hasnt_idle: Query<(), (With<Worker>, Without<Idle>)>,
    added_idle: Query<(), Added<Idle>>,
    removed_idle: RemovedComponents<Idle>,
    item_query: Query<&Children, With<IdleWorkers>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    if added_idle.is_empty() && removed_idle.is_empty() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some((mut text, mut text_color)) = text_iter.fetch_next() else {
        return;
    };

    let idle = has_idle.iter().count();
    let total = idle + hasnt_idle.iter().count();

    text.0 = format!("{}/{}", idle, total);
    text_color.0 = if idle != total {
        ui::TITLE_TEXT
    } else {
        bevy::color::palettes::css::RED.into()
    };
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
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    text.0 = format!("{fps:.1}");
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

    text.0 = format!("{current}/{max}");
}

fn update_stone(
    currency: Res<Currency>,
    item_query: Query<&Children, With<Stone>>,
    text_query: Query<Entity, With<Text>>,
    selected_tool: Res<SelectedTool>,
    mut writer: TextUiWriter,
) {
    if !currency.is_changed() && !selected_tool.is_changed() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many(children);
    let Some(text_entity) = text_iter.next() else {
        return;
    };

    *writer.text(text_entity, 0) = format!("{}", currency.stone);

    let price = DesignationKind::from(selected_tool.0).price();
    if price.stone > 0 {
        *writer.text(text_entity, 1) = format!("-{}", price.stone);
        writer.color(text_entity, 1).0 = if currency.stone >= price.stone {
            Color::srgb(0.0, 0.9, 0.0)
        } else {
            Color::srgb(0.9, 0.0, 0.0)
        };
    } else {
        writer.text(text_entity, 1).clear();
    }
}

fn update_metal(
    currency: Res<Currency>,
    item_query: Query<&Children, With<Metal>>,
    text_query: Query<Entity, With<Text>>,
    selected_tool: Res<SelectedTool>,
    mut writer: TextUiWriter,
) {
    if !currency.is_changed() && !selected_tool.is_changed() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many(children);
    let Some(text_entity) = text_iter.next() else {
        return;
    };

    *writer.text(text_entity, 0) = format!("{}", currency.metal);

    let price = DesignationKind::from(selected_tool.0).price();
    if price.metal > 0 {
        *writer.text(text_entity, 1) = format!("-{}", price.metal);
        writer.color(text_entity, 1).0 = if currency.metal >= price.metal {
            Color::srgb(0.0, 0.9, 0.0)
        } else {
            Color::srgb(0.9, 0.0, 0.0)
        };
    } else {
        writer.text(text_entity, 1).clear()
    }
}

fn update_crystal(
    currency: Res<Currency>,
    item_query: Query<&Children, With<Crystal>>,
    text_query: Query<Entity, With<Text>>,
    selected_tool: Res<SelectedTool>,
    mut writer: TextUiWriter,
) {
    if !currency.is_changed() && !selected_tool.is_changed() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many(children);
    let Some(text_entity) = text_iter.next() else {
        return;
    };

    *writer.text(text_entity, 0) = format!("{}", currency.crystal);

    let price = DesignationKind::from(selected_tool.0).price();
    if price.crystal > 0 {
        *writer.text(text_entity, 1) = format!("-{}", price.crystal);
        writer.color(text_entity, 1).0 = if currency.crystal >= price.crystal {
            Color::srgb(0.0, 0.9, 0.0)
        } else {
            Color::srgb(0.9, 0.0, 0.0)
        };
    } else {
        writer.text(text_entity, 1).clear();
    }
}

fn update_wave_count(
    waves: Option<Res<Waves>>,
    item_query: Query<&Children, With<WaveCount>>,
    mut text_query: Query<&mut Text>,
) {
    let Some(waves) = waves else {
        return;
    };

    if !waves.is_changed() {
        return;
    }

    let Ok(children) = item_query.get_single() else {
        return;
    };

    let mut text_iter = text_query.iter_many_mut(children);
    let Some(mut text) = text_iter.fetch_next() else {
        return;
    };

    // TODO color

    let num = waves.waves.len();
    let current = (waves.current + 1).min(num);

    text.0 = format!("{}/{}", current, num);
}

fn cleanup(mut commands: Commands, query: Query<Entity, With<HudRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
