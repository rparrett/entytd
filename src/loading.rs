use bevy::{
    asset::{LoadState, UntypedAssetId},
    prelude::*,
};

use crate::GameState;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadingAssets>()
            .add_systems(Update, wait.run_if(in_state(GameState::Loading)));
    }
}

#[derive(Default, Resource)]
pub struct LoadingAssets(pub Vec<UntypedAssetId>);

fn wait(
    loading: Res<LoadingAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if loading
        .0
        .iter()
        .all(|id| asset_server.get_load_state(*id) == Some(LoadState::Loaded))
    {
        next_state.set(GameState::Playing);
    }
}
