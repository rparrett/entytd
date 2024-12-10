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

fn update_sprites(mut query: Query<(&mut Sprite, &HitPoints), (Changed<HitPoints>, With<Home>)>) {
    for (mut sprite, hitpoints) in &mut query {
        if hitpoints.is_zero() {
            if let Some(ref mut texture_atlas) = sprite.texture_atlas {
                texture_atlas.index = TileKind::HomeDead.atlas_index();
            }
        }
    }
}
