use bevy::prelude::*;

use camera::CameraPlugin;
use loading::LoadingPlugin;
use map_loader::MapFileLoaderPlugin;
use tilemap::TilemapPlugin;

#[cfg(feature = "inspector")]
use {
    bevy::input::common_conditions::input_toggle_active,
    bevy_inspector_egui::quick::WorldInspectorPlugin,
};

mod camera;
mod loading;
mod map_loader;
mod tilemap;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            LoadingPlugin,
            CameraPlugin,
            TilemapPlugin,
            MapFileLoaderPlugin,
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
