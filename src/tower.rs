use bevy::prelude::*;

use crate::{
    designate_tool::Designations,
    tilemap::{AtlasHandle, TileEntities, TileKind, TilePos, Tilemap, SCALE},
    GameState,
};

pub struct TowerPlugin;
impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuildTowerEvent>()
            .add_systems(Update, build_tower.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Tower;

#[derive(Event, Debug)]
pub struct BuildTowerEvent(pub TilePos);

fn build_tower(
    mut commands: Commands,
    mut events: EventReader<BuildTowerEvent>,
    mut designations: ResMut<Designations>,
    mut tilemap_query: Query<(&mut Tilemap, &mut TileEntities)>,
    atlas_handle: Res<AtlasHandle>,
) {
    for event in events.read() {
        let Ok((mut tilemap, mut tile_entities)) = tilemap_query.get_single_mut() else {
            continue;
        };

        let world = tilemap.pos_to_world(event.0).extend(0.);

        let Some(tile_kind) = tilemap.get_mut(event.0) else {
            continue;
        };

        if !tile_kind.buildable() {
            continue;
        }

        let Some(maybe_tile_entity) = tile_entities.get_mut(event.0) else {
            continue;
        };

        // TODO subtract currency. If not enough currency, abort.

        if let Some(entity) = maybe_tile_entity.take() {
            commands.entity(entity).despawn();
        }

        let id = commands
            .spawn((
                SpriteSheetBundle {
                    texture_atlas: atlas_handle.0.clone(),
                    sprite: TextureAtlasSprite::new(TileKind::Tower.atlas_index()),
                    transform: Transform {
                        scale: SCALE.extend(1.),
                        translation: world,
                        ..default()
                    },
                    ..default()
                },
                Tower,
                TileKind::Tower,
                event.0.clone(),
            ))
            .id();

        *maybe_tile_entity = Some(id);
        *tile_kind = TileKind::Tower;

        if let Some(designation) = designations.0.remove(&event.0) {
            commands.entity(designation.indicator).despawn();
        }
    }
}
