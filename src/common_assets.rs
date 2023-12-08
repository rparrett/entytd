use bevy::prelude::*;

use crate::loading::LoadingAssets;

pub struct CommonAssetsPlugin;
impl Plugin for CommonAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CommonAssets>()
            .init_resource::<Sounds>();
    }
}

#[derive(Resource)]
pub struct Sounds {
    pub bgm: Handle<AudioSource>,
}
impl FromWorld for Sounds {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let bgm = asset_server.load("bgm.ogg");
        Sounds { bgm }
    }
}

#[derive(Resource)]
pub struct CommonAssets {
    pub ui_nine_slice: Handle<Image>,
    pub ui_nine_slice_selected: Handle<Image>,
    pub ui_nine_slice_container: Handle<Image>,
    pub ui_nine_slice_hovered: Handle<Image>,
}
impl FromWorld for CommonAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let ui_nine_slice = asset_server.load("ui_nine_slice.png");
        let ui_nine_slice_selected = asset_server.load("ui_nine_slice_selected.png");
        let ui_nine_slice_hovered = asset_server.load("ui_nine_slice_hovered.png");
        let ui_nine_slice_container = asset_server.load("ui_nine_slice_container.png");

        let mut loading_assets = world.resource_mut::<LoadingAssets>();

        loading_assets.0.push(ui_nine_slice.clone().into());
        loading_assets.0.push(ui_nine_slice_hovered.clone().into());
        loading_assets.0.push(ui_nine_slice_selected.clone().into());
        loading_assets
            .0
            .push(ui_nine_slice_container.clone().into());

        CommonAssets {
            ui_nine_slice,
            ui_nine_slice_selected,
            ui_nine_slice_hovered,
            ui_nine_slice_container,
        }
    }
}
