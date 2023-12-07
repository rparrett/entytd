use bevy::{asset::AssetMetaCheck, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};

use bevy_nine_slice_ui::NineSlicePlugin;
use camera::CameraPlugin;
use common_assets::CommonAssetsPlugin;
use cursor::CursorPlugin;
use designate_tool::DesignateToolPlugin;
use enemy::EnemyPlugin;
use home::HomePlugin;
use hud::HudPlugin;
use level::LevelPlugin;
use loading::LoadingPlugin;
use map_loader::MapFileLoaderPlugin;
use movement::MovementPlugin;
use pathfinding::PathfindingPlugin;
use radio_button::RadioButtonPlugin;
use spawner::SpawnerPlugin;
use stone::StonePlugin;
use tilemap::TilemapPlugin;
use tool_selector::ToolSelectorPlugin;
use waves::WavesPlugin;
use worker::WorkerPlugin;

#[cfg(feature = "inspector")]
use {
    bevy::input::common_conditions::input_toggle_active,
    bevy_inspector_egui::quick::WorldInspectorPlugin,
};

mod camera;
mod common_assets;
mod cursor;
mod designate_tool;
mod enemy;
mod hit_points;
mod home;
mod hud;
mod level;
mod loading;
mod map_loader;
mod movement;
mod pathfinding;
mod radio_button;
mod spawner;
mod stone;
mod tilemap;
mod tool_selector;
mod waves;
mod worker;

fn main() {
    let mut app = App::new();

    // Workaround for Bevy attempting to load .meta files in wasm builds. On itch,
    // the CDN servers HTTP 403 errors instead of 404 when files don't exists, which
    // causes Bevy to break.
    app.insert_resource(AssetMetaCheck::Never);

    app.add_plugins((
        DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
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
        CommonAssetsPlugin,
        WorkerPlugin,
        MovementPlugin,
        StonePlugin,
    ));

    app.add_plugins((
        RadioButtonPlugin,
        ToolSelectorPlugin,
        DesignateToolPlugin,
        CursorPlugin,
        HudPlugin,
    ));

    app.add_plugins(NineSlicePlugin::default());

    #[cfg(feature = "inspector")]
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
    );

    app.insert_resource(Msaa::Off).add_state::<GameState>();

    app.run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
