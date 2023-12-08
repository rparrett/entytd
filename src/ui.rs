use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{common_assets::CommonAssets, radio_button::RadioButton};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_style);
    }
}

pub fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut NineSliceTexture, Option<&RadioButton>),
        (Changed<Interaction>, With<Button>),
    >,
    assets: Res<CommonAssets>,
) {
    for (interaction, mut texture, maybe_radio) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *texture = NineSliceTexture::from_image(assets.ui_nine_slice_selected.clone());
            }
            Interaction::Hovered => {
                if !maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    *texture = NineSliceTexture::from_image(assets.ui_nine_slice_hovered.clone());
                }
            }
            Interaction::None => {
                *texture = if maybe_radio.map_or_else(|| false, |radio| radio.selected) {
                    NineSliceTexture::from_image(assets.ui_nine_slice_selected.clone())
                } else {
                    NineSliceTexture::from_image(assets.ui_nine_slice.clone())
                };
            }
        }
    }
}
