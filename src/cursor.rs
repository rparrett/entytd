use bevy::prelude::*;

use crate::tilemap::{TilePos, Tilemap, TilemapHandle};

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .init_resource::<CursorSnapped>()
            .add_systems(Update, cursor);
    }
}

#[derive(Resource, Default)]
pub struct Cursor {
    pub viewport_pos: Vec2,
    pub world_pos: Vec2,
}
#[derive(Resource, Default)]
pub struct CursorSnapped {
    pub world_pos: Option<Vec2>,
    pub tile_pos: Option<TilePos>,
}

fn cursor(
    mut events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, Ref<GlobalTransform>)>,
    maybe_tilemap_handle: Option<Res<TilemapHandle>>,
    tilemaps: Res<Assets<Tilemap>>,
    mut cursor: ResMut<Cursor>,
    mut cursor_snapped: ResMut<CursorSnapped>,
    added_window: Query<&Window, Added<Window>>,
) {
    let mut changed = false;
    for event in events.read() {
        cursor.viewport_pos = event.position;
        changed = true;
    }

    for window in &added_window {
        cursor.viewport_pos = window
            .cursor_position()
            .unwrap_or_else(|| Vec2::new(window.width(), window.height()) / 2.);
        changed = true;
    }

    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    if !changed && !camera_transform.is_changed() {
        return;
    }

    let Some(world) = camera.viewport_to_world_2d(&camera_transform, cursor.viewport_pos) else {
        return;
    };

    cursor.world_pos = world;

    let Some(tilemap_handle) = maybe_tilemap_handle else {
        return;
    };

    let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    let tile = tilemap.world_to_pos(world);
    let snapped = tilemap.pos_to_world(tile);

    cursor_snapped.tile_pos = Some(tile);
    cursor_snapped.world_pos = Some(snapped);
}
