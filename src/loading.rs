use bevy::{
    asset::{LoadState, UntypedAssetId},
    prelude::*,
};
use bevy_nine_slice_ui::NineSliceTexture;
use bevy_pipelines_ready::{PipelinesReady, PipelinesReadyPlugin};
use strum::IntoEnumIterator;

use crate::{
    enemy::EnemyKind,
    tilemap::{AtlasHandle, TileKind},
    ui::UiAssets,
    util::cleanup,
    GameState,
};

#[cfg(not(target_arch = "wasm32"))]
const EXPECTED_PIPELINES: usize = 7;
#[cfg(target_arch = "wasm32")]
const EXPECTED_PIPELINES: usize = 7;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PipelinesReadyPlugin)
            .init_resource::<LoadingResources>()
            .init_resource::<LoadingAssets>()
            .add_systems(
                Update,
                (init_loading_scene, animate_loading_scene, wait)
                    .run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                Update,
                log_pipelines.run_if(resource_changed::<PipelinesReady>()),
            )
            .add_systems(OnExit(GameState::Loading), cleanup::<LoadingScene>);
    }
}

#[derive(Component)]
struct LoadingScene;

#[derive(Component)]
pub struct LoadingImage {
    frames: Vec<usize>,
    timer: Timer,
    index: usize,
}

#[derive(Default, Resource)]
pub struct LoadingAssets(pub Vec<UntypedAssetId>);
#[derive(Default, Resource)]
pub struct LoadingResources(pub usize);

fn wait(
    loading: Res<LoadingAssets>,
    loading_resources: Res<LoadingResources>,
    asset_server: Res<AssetServer>,
    pipelines: Res<PipelinesReady>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let assets = loading
        .0
        .iter()
        .all(|id| asset_server.get_load_state(*id) == Some(LoadState::Loaded));

    let resources = loading_resources.0 == 0;

    let pipelines = pipelines.get() >= EXPECTED_PIPELINES;

    if assets && resources && pipelines {
        info!("Advancing to GameState::MainMenu");

        next_state.set(GameState::MainMenu);
    }
}

fn init_loading_scene(
    mut commands: Commands,
    maybe_atlas_handle: Option<Res<AtlasHandle>>,
    common: Res<UiAssets>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    let Some(atlas_handle) = maybe_atlas_handle else {
        return;
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                ..default()
            },
            NineSliceTexture::from_image(common.nine_slice.clone()),
            LoadingScene,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section("Loading...", TextStyle::default()),
                ..default()
            });
            parent.spawn((
                AtlasImageBundle {
                    texture_atlas_image: UiTextureAtlasImage {
                        index: EnemyKind::Ent.atlas_index(),
                        ..default()
                    },
                    texture_atlas: atlas_handle.0.clone(),
                    ..default()
                },
                LoadingImage {
                    frames: TileKind::iter().map(|t| t.atlas_index()).collect(),
                    timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                    index: 0,
                },
            ));
        });

    *done = true;
}

fn animate_loading_scene(
    mut query: Query<(&mut UiTextureAtlasImage, &mut LoadingImage)>,
    time: Res<Time>,
) {
    for (mut image, mut anim) in &mut query {
        anim.timer.tick(time.delta());
        if !anim.timer.just_finished() {
            continue;
        }

        anim.index += 1;
        if anim.index > anim.frames.len() - 1 {
            anim.index = 0;
        }

        image.index = anim.frames[anim.index];
    }
}

fn log_pipelines(pipelines: Res<PipelinesReady>) {
    info!("Pipelines: {}/{}", pipelines.get(), EXPECTED_PIPELINES);
}
