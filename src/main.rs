use bevy::prelude::*;

use bevy_nine_slice_ui::NineSlicePlugin;
use camera::CameraPlugin;
use designate_tool::DesignateToolPlugin;
use enemy::EnemyPlugin;
use home::HomePlugin;
use level::LevelPlugin;
use loading::LoadingPlugin;
use map_loader::MapFileLoaderPlugin;
use pathfinding::PathfindingPlugin;
use radio_button::RadioButtonPlugin;
use spawner::SpawnerPlugin;
use tilemap::TilemapPlugin;
use tool_selector::ToolSelectorPlugin;
use waves::WavesPlugin;

#[cfg(feature = "inspector")]
use {
    bevy::input::common_conditions::input_toggle_active,
    bevy_inspector_egui::quick::WorldInspectorPlugin,
};

mod camera;
mod designate_tool;
mod enemy;
mod hit_points;
mod home;
mod level;
mod loading;
mod map_loader;
mod pathfinding;
mod radio_button;
mod spawner;
mod tilemap;
mod tool_selector;
mod util;
mod waves;

fn main() {
    App::new()
        .add_plugins((
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
            LoadingPlugin,
            CameraPlugin,
            TilemapPlugin,
            MapFileLoaderPlugin,
            SpawnerPlugin,
            HomePlugin,
            WavesPlugin,
            EnemyPlugin,
            LevelPlugin,
            RadioButtonPlugin,
            ToolSelectorPlugin,
            DesignateToolPlugin,
            PathfindingPlugin,
            NineSlicePlugin::default(),
            #[cfg(feature = "inspector")]
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        ))
        .insert_resource(Msaa::Off)
        .add_state::<GameState>()
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}
