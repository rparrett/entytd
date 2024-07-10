use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceUiTexture;

use crate::{loading::LoadingAssets, radio_button::RadioButton};

pub const BUTTON_TEXT: Color = Color::srgb(0.9, 0.9, 0.9);
pub const TITLE_TEXT: Color = Color::srgb(0.9, 0.9, 0.9);

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiAssets>()
            .add_systems(Update, button_style);
    }
}

#[derive(Resource)]
pub struct UiAssets {
    pub nine_button: Handle<Image>,
    pub nine_button_selected: Handle<Image>,
    pub nine_button_hovered: Handle<Image>,
    pub nine_panel: Handle<Image>,
    pub nine_panel_warning: Handle<Image>,
    pub nine_panel_info: Handle<Image>,
    pub range_indicator_24: Handle<Image>,
}
impl FromWorld for UiAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();

        let nine_button = asset_server.load("ui/nine_button.png");
        let nine_button_selected = asset_server.load("ui/nine_button_selected.png");
        let nine_button_hovered = asset_server.load("ui/nine_button_hovered.png");
        let nine_panel = asset_server.load("ui/nine_panel.png");
        let nine_panel_warning = asset_server.load("ui/nine_panel_warning.png");
        let nine_panel_info = asset_server.load("ui/nine_panel_info.png");
        let range_indicator_24 = asset_server.load("ui/range_indicator_24.png");

        let mut loading_assets = world.resource_mut::<LoadingAssets>();

        loading_assets.0.push(nine_button.id().into());
        loading_assets.0.push(nine_button_selected.id().into());
        loading_assets.0.push(nine_button_hovered.id().into());
        loading_assets.0.push(nine_panel.id().into());
        loading_assets.0.push(nine_panel_warning.id().into());
        loading_assets.0.push(nine_panel_info.id().into());
        loading_assets.0.push(range_indicator_24.id().into());

        UiAssets {
            nine_button,
            nine_button_selected,
            nine_button_hovered,
            nine_panel,
            nine_panel_warning,
            nine_panel_info,
            range_indicator_24,
        }
    }
}

pub fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut NineSliceUiTexture, Option<&RadioButton>),
        (Changed<Interaction>, With<Button>),
    >,
    assets: Res<UiAssets>,
) {
    for (interaction, mut texture, maybe_radio) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *texture = NineSliceUiTexture::from_image(assets.nine_button_selected.clone());
            }
            Interaction::Hovered => {
                if !maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    *texture = NineSliceUiTexture::from_image(assets.nine_button_hovered.clone());
                }
            }
            Interaction::None => {
                *texture = if maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    NineSliceUiTexture::from_image(assets.nine_button_selected.clone())
                } else {
                    NineSliceUiTexture::from_image(assets.nine_button.clone())
                };
            }
        }
    }
}
