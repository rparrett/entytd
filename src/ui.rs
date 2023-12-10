use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{loading::LoadingAssets, radio_button::RadioButton};

pub const BUTTON_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::rgb(0.9, 0.9, 0.9);

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .add_systems(Update, button_style);
    }
}

#[derive(Resource)]
pub struct UiAssets {
    pub nine_slice: Handle<Image>,
    pub nine_slice_selected: Handle<Image>,
    pub nine_slice_container: Handle<Image>,
    pub nine_slice_hovered: Handle<Image>,
    pub range_indicator_24: Handle<Image>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let nine_slice = asset_server.load("ui_nine_slice.png");
        let nine_slice_selected = asset_server.load("ui_nine_slice_selected.png");
        let nine_slice_hovered = asset_server.load("ui_nine_slice_hovered.png");
        let nine_slice_container = asset_server.load("ui_nine_slice_container.png");
        let range_indicator_24 = asset_server.load("range_indicator_24.png");

        let mut loading_assets = world.resource_mut::<LoadingAssets>();

        loading_assets.0.push(nine_slice.clone().into());
        loading_assets.0.push(nine_slice_hovered.clone().into());
        loading_assets.0.push(nine_slice_selected.clone().into());
        loading_assets.0.push(nine_slice_container.clone().into());
        loading_assets.0.push(range_indicator_24.clone().into());

        UiAssets {
            nine_slice,
            nine_slice_selected,
            nine_slice_hovered,
            nine_slice_container,
            range_indicator_24,
        }
    }
}

pub fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut NineSliceTexture, Option<&RadioButton>),
        (Changed<Interaction>, With<Button>),
    >,
    assets: Res<UiAssets>,
) {
    for (interaction, mut texture, maybe_radio) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *texture = NineSliceTexture::from_image(assets.nine_slice_selected.clone());
            }
            Interaction::Hovered => {
                if !maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    *texture = NineSliceTexture::from_image(assets.nine_slice_hovered.clone());
                }
            }
            Interaction::None => {
                *texture = if maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    NineSliceTexture::from_image(assets.nine_slice_selected.clone())
                } else {
                    NineSliceTexture::from_image(assets.nine_slice.clone())
                };
            }
        }
    }
}
