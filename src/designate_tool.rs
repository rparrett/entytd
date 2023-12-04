use bevy::prelude::*;

use crate::{
    cursor::Cursor,
    tool_selector::{SelectedTool, Tool},
    GameState,
};

pub struct DesignateToolPlugin;
impl Plugin for DesignateToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (move_cursor, show_cursor).run_if(in_state(GameState::Playing)),
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

fn move_cursor(cursor: Res<Cursor>, mut query: Query<&mut Transform, With<DesignateToolCursor>>) {
    if !cursor.is_changed() {
        return;
    }

    for mut transform in &mut query {
        let Some(snapped) = cursor.world_pos_snapped else {
            continue;
        };

        transform.translation.x = snapped.x;
        transform.translation.y = snapped.y;
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
