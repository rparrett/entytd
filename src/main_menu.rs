use crate::{
    level::LevelConfig,
    loading::LoadingAssets,
    settings::{DifficultySetting, MusicSetting, ParticlesSetting, SfxSetting},
    sound::{MusicController, SoundAssets},
    tilemap::{AtlasHandle, Map, TileEntities, TilemapBundle, TilemapHandle},
    ui::{slice_image_mode, UiAssets, BUTTON_TEXT, TITLE_TEXT},
    GameState,
};
use bevy::{
    audio::{AudioSink, Volume},
    prelude::*,
};
use grid::Grid;

pub struct MainMenuPlugin;
impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuAssets>()
            .add_systems(OnEnter(GameState::MainMenu), (setup_menu, init_background))
            .add_systems(
                Update,
                (
                    sfx_volume,
                    music_volume,
                    play_button,
                    sfx_button,
                    music_button,
                    difficulty_button,
                    particles_button,
                )
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_background);
    }
}

#[derive(Resource)]
pub struct MainMenuAssets {
    pub map: Handle<Map>,
    pub level: Handle<LevelConfig>,
}
impl FromWorld for MainMenuAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let level = asset_server.load("levels/menu.level.ron");
        let map = asset_server.load("levels/menu.map.png");

        let mut loading_assets = world.resource_mut::<LoadingAssets>();
        loading_assets.0.push(level.id().into());
        loading_assets.0.push(map.id().into());

        MainMenuAssets { level, map }
    }
}

#[derive(Component)]
struct PlayButton;
#[derive(Component)]
struct MusicSettingButton;
#[derive(Component)]
struct MusicSettingButtonText;
#[derive(Component)]
struct SfxSettingButton;
#[derive(Component)]
struct SfxSettingButtonText;
#[derive(Component)]
struct DifficultySettingButton;
#[derive(Component)]
struct DifficultySettingButtonText;
#[derive(Component)]
struct ParticlesSettingButton;
#[derive(Component)]
struct ParticlesSettingButtonText;

fn setup_menu(
    mut commands: Commands,
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    difficulty: Res<DifficultySetting>,
    particles: Res<ParticlesSetting>,
    ui_assets: Res<UiAssets>,
) {
    let button_node = (
        Node {
            width: Val::Px(250.0),
            height: Val::Px(45.0),
            margin: UiRect::all(Val::Px(5.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode {
            image: ui_assets.nine_button.clone(),
            image_mode: slice_image_mode(),
            ..default()
        },
    );
    let button_text_style = (
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(BUTTON_TEXT),
    );
    let title_text_style = (
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(TITLE_TEXT),
    );
    let subtitle_text_style = (
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(TITLE_TEXT),
    );

    let container = commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            ImageNode {
                image: ui_assets.nine_panel.clone(),
                image_mode: slice_image_mode(),
                ..default()
            },
            DespawnOnExit(GameState::MainMenu),
        ))
        .id();

    let title = commands
        .spawn((
            Text::new("Enty TD"),
            title_text_style,
            Node {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    let play_button = commands
        .spawn((Button, button_node.clone(), PlayButton))
        .with_children(|parent| {
            parent.spawn((Text::new("Play"), button_text_style.clone()));
        })
        .id();

    let audio_settings_title = commands
        .spawn((
            Text::new("Audio"),
            subtitle_text_style.clone(),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let difficulty_title = commands
        .spawn((
            Text::new("Difficulty"),
            subtitle_text_style.clone(),
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let graphics_title = commands
        .spawn((
            Text::new("Graphics"),
            subtitle_text_style,
            Node {
                margin: UiRect::all(Val::Px(10.0)),
                ..default()
            },
        ))
        .id();

    let particles_button = commands
        .spawn((Button, button_node.clone(), ParticlesSettingButton))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{}", *particles)),
                button_text_style.clone(),
                ParticlesSettingButtonText,
            ));
        })
        .id();

    let difficulty_button = commands
        .spawn((Button, button_node.clone(), DifficultySettingButton))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{}", *difficulty)),
                button_text_style.clone(),
                DifficultySettingButtonText,
            ));
        })
        .id();

    let sfx_button = commands
        .spawn((Button, button_node.clone(), SfxSettingButton))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Sfx {}%", **sfx)),
                button_text_style.clone(),
                SfxSettingButtonText,
            ));
        })
        .id();

    let music_button = commands
        .spawn((Button, button_node, MusicSettingButton))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("Music {}%", **music)),
                button_text_style,
                MusicSettingButtonText,
            ));
        })
        .id();

    commands.entity(container).add_children(&[
        title,
        play_button,
        difficulty_title,
        difficulty_button,
        audio_settings_title,
        sfx_button,
        music_button,
        graphics_title,
        particles_button,
    ]);
}

fn play_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

fn sfx_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SfxSettingButton>)>,
    mut text_query: Query<&mut Text, With<SfxSettingButtonText>>,
    mut sfx_setting: ResMut<SfxSetting>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if **sfx_setting == 0 {
                **sfx_setting = 100;
            } else {
                **sfx_setting -= 10;
            }

            for mut text in text_query.iter_mut() {
                text.0 = format!("Sfx {}%", **sfx_setting);
            }
        }
    }
}

fn music_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<MusicSettingButton>)>,
    mut text_query: Query<&mut Text, With<MusicSettingButtonText>>,
    mut music_setting: ResMut<MusicSetting>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if **music_setting == 0 {
                **music_setting = 100;
            } else {
                **music_setting -= 10;
            }

            for mut text in text_query.iter_mut() {
                text.0 = format!("Music {}%", **music_setting);
            }
        }
    }
}

fn difficulty_button(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<DifficultySettingButton>),
    >,
    mut text_query: Query<&mut Text, With<DifficultySettingButtonText>>,
    mut difficulty_setting: ResMut<DifficultySetting>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            *difficulty_setting = difficulty_setting.next();

            for mut text in text_query.iter_mut() {
                text.0 = format!("{}", *difficulty_setting);
            }
        }
    }
}

fn particles_button(
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<ParticlesSettingButton>),
    >,
    mut text_query: Query<&mut Text, With<ParticlesSettingButtonText>>,
    mut particles_setting: ResMut<ParticlesSetting>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            *particles_setting = particles_setting.next();

            for mut text in text_query.iter_mut() {
                text.0 = format!("{}", *particles_setting);
            }
        }
    }
}

fn init_background(
    mut commands: Commands,
    atlas_handle: Res<AtlasHandle>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    assets: Res<MainMenuAssets>,
    maps: Res<Assets<Map>>,
) {
    let tiles = maps.get(&assets.map).unwrap().clone();
    let entities = TileEntities(Grid::new(tiles.0.rows(), tiles.0.cols()));

    commands.spawn(TilemapBundle {
        tilemap_handle: TilemapHandle(assets.map.clone()),
        atlas_handle: atlas_handle.clone(),
        tiles,
        entities,
    });

    for mut transform in &mut camera_query {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    }
}

fn cleanup_background(mut commands: Commands, query: Query<(Entity, &TileEntities)>) {
    for (entity, entities) in &query {
        commands.entity(entity).despawn();
        for entity in entities.0.iter().flatten() {
            commands.entity(*entity).despawn();
        }
    }
}

fn sfx_volume(
    mut commands: Commands,
    sfx_setting: Res<SfxSetting>,
    sound_assets: Res<SoundAssets>,
) {
    // Do not run when SfxSetting is first added by SavePlugin
    if !sfx_setting.is_changed() || sfx_setting.is_added() {
        return;
    }

    commands.spawn((
        AudioPlayer(sound_assets.pickaxe.clone()),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(**sfx_setting as f32 / 100.)),
    ));
}

fn music_volume(
    music_setting: Res<MusicSetting>,
    mut music_query: Query<&mut AudioSink, With<MusicController>>,
) {
    // Do not run when MusicSetting is first added by SavePlugin
    if !music_setting.is_changed() || music_setting.is_added() {
        return;
    }

    if let Ok(mut sink) = music_query.single_mut() {
        sink.set_volume(Volume::Linear(**music_setting as f32 / 100.));
    }
}
