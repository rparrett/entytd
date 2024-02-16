// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{asset::AssetMetaCheck, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use bevy_nine_slice_ui::NineSliceUiPlugin;
use camera::CameraPlugin;
use currency::CurrencyPlugin;
use cursor::CursorPlugin;
use designate_tool::DesignateToolPlugin;
use enemy::EnemyPlugin;
use game::GamePlugin;
use game_over::GameOverPlugin;
use home::HomePlugin;
use hud::HudPlugin;
use level::LevelPlugin;
use loading::LoadingPlugin;
use main_menu::MainMenuPlugin;
use map_loader::MapFileLoaderPlugin;
use movement::MovementPlugin;
use particle::ParticlePlugin;
use pathfinding::PathfindingPlugin;
use radio_button::RadioButtonPlugin;
use save::SavePlugin;
use sound::MusicPlugin;
use spawner::SpawnerPlugin;
use stats::StatsPlugin;
use stone::StonePlugin;
use tilemap::TilemapPlugin;
use tool_selector::ToolSelectorPlugin;
use tower::TowerPlugin;
use tutorial::TutorialPlugin;
use ui::UiPlugin;
use waves::WavesPlugin;
use worker::WorkerPlugin;

#[cfg(feature = "inspector")]
use {
    bevy::input::common_conditions::input_toggle_active,
    bevy_inspector_egui::quick::WorldInspectorPlugin,
};

mod camera;
mod currency;
mod cursor;
mod designate_tool;
mod enemy;
mod game;
mod game_over;
mod hit_points;
mod home;
mod hud;
mod layer;
mod level;
mod loading;
mod main_menu;
mod map_loader;
mod movement;
mod particle;
mod pathfinding;
mod radio_button;
mod save;
mod settings;
mod sound;
mod spawner;
mod stats;
mod stone;
mod tilemap;
mod tool_selector;
mod tower;
mod tutorial;
mod ui;
mod util;
mod waves;
mod worker;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    GameOver,
}

fn main() {
    let mut app = App::new();

    // Workaround for Bevy attempting to load .meta files in wasm builds. On itch,
    // the CDN serves HTTP 403 errors instead of 404 when files don't exist, which
    // causes Bevy to break.
    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Enty TD".to_string(),
                    #[cfg(feature = "recording")]
                    decorations: false,
                    ..default()
                }),
                ..default()
            }),
        FrameTimeDiagnosticsPlugin,
    ));

    app.add_plugins((
        LoadingPlugin,
        CameraPlugin,
        TilemapPlugin,
        MapFileLoaderPlugin,
        SpawnerPlugin,
        HomePlugin,
        WavesPlugin,
        EnemyPlugin,
        LevelPlugin,
        PathfindingPlugin,
        WorkerPlugin,
        MovementPlugin,
        StonePlugin,
        CurrencyPlugin,
    ));
    app.add_plugins((
        TowerPlugin,
        ParticlePlugin,
        SavePlugin,
        MusicPlugin,
        GameOverPlugin,
        GamePlugin,
        StatsPlugin,
    ));

    app.add_plugins((
        RadioButtonPlugin,
        ToolSelectorPlugin,
        DesignateToolPlugin,
        CursorPlugin,
        HudPlugin,
        MainMenuPlugin,
        UiPlugin,
        TutorialPlugin,
    ));

    app.add_plugins(NineSliceUiPlugin::default());

    #[cfg(feature = "inspector")]
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
    );

    app.insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::BLACK))
        .add_state::<GameState>();

    app.run();
}
