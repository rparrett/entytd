use bevy::prelude::*;

use bevy_nine_slice_ui::NineSlicePlugin;
use camera::CameraPlugin;
use enemy::EnemyPlugin;
use home::HomePlugin;
use level::LevelPlugin;
use loading::LoadingPlugin;
use map_loader::MapFileLoaderPlugin;
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
mod enemy;
mod hit_points;
mod home;
mod level;
mod loading;
mod map_loader;
mod spawner;
mod tilemap;
mod tool_selector;
mod util;
mod waves;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            LoadingPlugin,
            CameraPlugin,
            TilemapPlugin,
            MapFileLoaderPlugin,
            SpawnerPlugin,
            HomePlugin,
            WavesPlugin,
            EnemyPlugin,
            LevelPlugin,
            ToolSelectorPlugin,
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
