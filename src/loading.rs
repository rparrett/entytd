use bevy::{
    asset::{LoadState, UntypedAssetId},
    prelude::*,
};

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingResources>()
            .init_resource::<LoadingAssets>()
            .add_systems(Update, wait.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Default, Resource)]
pub struct LoadingAssets(pub Vec<UntypedAssetId>);
#[derive(Default, Resource)]
pub struct LoadingResources(pub usize);

fn wait(
    loading: Res<LoadingAssets>,
    loading_resources: Res<LoadingResources>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let assets = loading
        .0
        .iter()
        .all(|id| asset_server.get_load_state(*id) == Some(LoadState::Loaded));

    let resources = loading_resources.0 == 0;

    if assets && resources {
        info!("Advancing to GameState::Playing");

        next_state.set(GameState::Playing);
    }
}
