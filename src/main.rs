//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use camera::CameraPlugin;
use tilemap::{TileAtlas, Tilemap, TilemapPlugin};

mod camera;
mod tilemap;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            CameraPlugin,
            TilemapPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, atlas: Res<TileAtlas>) {
    let mut map = Tilemap::new_random(54, 30);
    map.spawn(&mut commands, atlas.0.clone());
}
