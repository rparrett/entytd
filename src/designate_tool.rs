use bevy::prelude::*;

use crate::{
    tilemap::{Tilemap, TilemapHandle},
    tool_selector::{SelectedTool, Tool},
    GameState,
};

pub struct DesignateToolPlugin;
impl Plugin for DesignateToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (cursor, show_cursor).run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnEnter(GameState::Playing), init_cursor);
    }
}

#[derive(Component)]
struct DesignateToolCursor;

fn init_cursor(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(crate::tilemap::SCALE * crate::tilemap::TILE_SIZE),
                color: Color::AQUAMARINE.with_a(0.5),
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        DesignateToolCursor,
    ));
}

fn cursor(
    mut events: EventReader<CursorMoved>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform, With<DesignateToolCursor>>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    // TODO we need to update this when the camera moves as well.

    for event in events.read() {
        let Ok((camera, camera_transform)) = camera_query.get_single() else {
            continue;
        };

        let Some(world) = camera.viewport_to_world_2d(camera_transform, event.position) else {
            continue;
        };

        let Ok(mut cursor_transform) = query.get_single_mut() else {
            continue;
        };

        let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
            continue;
        };

        let tile = tilemap.world_to_pos(world);
        let snapped = tilemap.pos_to_world(tile);

        cursor_transform.translation.x = snapped.x;
        cursor_transform.translation.y = snapped.y;
    }
}

fn show_cursor(
    selected_tool: Res<SelectedTool>,
    mut query: Query<&mut Visibility, With<DesignateToolCursor>>,
) {
    if !selected_tool.is_changed() {
        return;
    };

    let Ok(mut visibility) = query.get_single_mut() else {
        return;
    };

    *visibility = match selected_tool.0 {
        Tool::Designate => Visibility::Visible,
        _ => Visibility::Hidden,
    };
}
