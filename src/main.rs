//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use camera::CameraPlugin;
use loading::LoadingPlugin;
use map_loader::MapFileLoaderPlugin;
use tilemap::{TileAtlas, Tilemap, TilemapPlugin};

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
        ))
        .add_state::<GameState>()
        .add_systems(Startup, setup)
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

fn setup(mut commands: Commands, atlas: Res<TileAtlas>) {
    // let mut map = Tilemap::new_random(108, 60);
    // map.spawn(&mut commands, atlas.0.clone());
    // commands.insert_resource(map);
}
