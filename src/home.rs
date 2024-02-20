use bevy::prelude::*;

use crate::{hit_points::HitPoints, tilemap::TileKind, GameState};

pub struct HomePlugin;
impl Plugin for HomePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_sprites.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Home;

fn update_sprites(
    mut query: Query<(&mut TextureAtlas, &HitPoints), (Changed<HitPoints>, With<Home>)>,
) {
    for (mut atlas, hitpoints) in &mut query {
        if hitpoints.is_zero() {
            atlas.index = TileKind::HomeDead.atlas_index();
        }
    }
}
