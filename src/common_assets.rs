use bevy::prelude::*;

use crate::loading::LoadingAssets;

pub struct CommonAssetsPlugin;
impl Plugin for CommonAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommonAssets>();
    }
}

#[derive(Resource)]
pub struct CommonAssets {
    pub ui_nine_slice: Handle<Image>,
    pub ui_nine_slice_selected: Handle<Image>,
}
impl FromWorld for CommonAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let common = CommonAssets {
            ui_nine_slice: asset_server.load("ui_nine_slice.png"),
            ui_nine_slice_selected: asset_server.load("ui_nine_slice_selected.png"),
        };

        let mut loading_assets = world.resource_mut::<LoadingAssets>();

        loading_assets.0.push(common.ui_nine_slice.clone().into());
        loading_assets
            .0
            .push(common.ui_nine_slice_selected.clone().into());

        common
    }
}
