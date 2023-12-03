use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::Deserialize;

use crate::{
    loading::{LoadingAssets, LoadingResources},
    waves::Waves,
    GameState,
};

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<LevelConfig>::new(&["level.ron"]))
            .add_systems(OnEnter(GameState::Loading), queue_load)
            .add_systems(Update, check_load.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Asset, TypePath, Deserialize)]
pub struct LevelConfig {
    pub map: String,
    pub waves: Waves,
}

#[derive(Resource)]
pub struct LevelHandle(pub Handle<LevelConfig>);

fn queue_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut loading_resources: ResMut<LoadingResources>,
) {
    let handle = asset_server.load("1.level.ron");
    loading_assets.0.push(handle.clone().into());
    commands.insert_resource(LevelHandle(handle));
    loading_resources.0 += 1;
}

fn check_load(
    mut loading_resources: ResMut<LoadingResources>,
    level_handle: Option<Res<LevelHandle>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }

    if level_handle.is_some() {
        loading_resources.0 -= 1;
        *done = true;
    }
}
