use bevy::{prelude::*, utils::HashSet};

use crate::{
    cursor::{Cursor, CursorSnapped},
    loading::LoadingResources,
    tilemap::{TileKind, TilePos, Tilemap, TilemapHandle},
    tool_selector::{SelectedTool, Tool},
    GameState,
};

const DESIGNATE_OK: Color = Color::rgba(0., 1.0, 1.0, 0.5);
const DESIGNATE_NOT_OK: Color = Color::rgba(1.0, 0.0, 0.0, 0.5);

pub struct DesignateToolPlugin;
impl Plugin for DesignateToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DesignationToolState>();
        app.add_systems(
            Update,
            (move_cursor, show_cursor).run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            (update_tool_state, designate)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            init_designations.run_if(in_state(GameState::Loading)),
        );
        app.add_systems(OnEnter(GameState::Playing), init_cursor);
    }
}

#[derive(Component)]
struct DesignateToolCursor;

#[derive(Copy, Clone)]
enum DesignationKind {
    Dig,
    BuildTower,
    Dance,
}

#[derive(Clone)]
struct Designation {
    kind: DesignationKind,
    indicator: Entity,
}

#[derive(Component)]
struct DesignationMarker;

#[derive(Resource)]
struct Designations(Vec<Vec<Option<Designation>>>);

#[derive(Resource, Default)]
struct DesignationToolState {
    active: bool,
    removing: bool,
    touched: HashSet<TilePos>,
}

fn init_designations(
    mut commands: Commands,
    mut loading_resources: ResMut<LoadingResources>,
    maybe_designations: Option<Res<Designations>>,
    maybe_tilemap_handle: Option<Res<TilemapHandle>>,
    tilemaps: Res<Assets<Tilemap>>,
    mut queued: Local<bool>,
) {
    if maybe_designations.is_some() {
        return;
    }

    if !*queued {
        loading_resources.0 += 1;
        *queued = true;
    }

    let Some(tilemap_handle) = maybe_tilemap_handle else {
        return;
    };

    let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    commands.insert_resource(Designations(vec![
        vec![None; tilemap.height];
        tilemap.width
    ]));
    loading_resources.0 -= 1;
}

fn init_cursor(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(crate::tilemap::SCALE * crate::tilemap::TILE_SIZE),
                color: DESIGNATE_OK,
                ..default()
            },
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        DesignateToolCursor,
    ));
}

fn move_cursor(
    selected_tool: Res<SelectedTool>,
    cursor_snapped: Res<CursorSnapped>,
    mut query: Query<(&mut Transform, &mut Sprite), With<DesignateToolCursor>>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    if !cursor_snapped.is_changed() {
        return;
    }

    for (mut transform, mut sprite) in &mut query {
        let Some(snapped) = cursor_snapped.world_pos else {
            continue;
        };

        let Some(tile_pos) = cursor_snapped.tile_pos else {
            continue;
        };

        transform.translation.x = snapped.x;
        transform.translation.y = snapped.y;

        let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
            return;
        };

        match (selected_tool.0, &tilemap.tiles[tile_pos.x][tile_pos.y]) {
            (Tool::Dig, TileKind::Stone) => sprite.color = DESIGNATE_OK,
            _ => sprite.color = DESIGNATE_NOT_OK,
        };
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
        Tool::Dig => Visibility::Visible,
        Tool::BuildTower => Visibility::Visible,
        _ => Visibility::Hidden,
    };
}

fn update_tool_state(
    buttons: Res<Input<MouseButton>>,
    cursor_snapped: Res<CursorSnapped>,
    mut tool_state: ResMut<DesignationToolState>,
    designations: Res<Designations>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Some(tile_pos) = cursor_snapped.tile_pos else {
            return;
        };

        tool_state.active = true;
        if designations.0[tile_pos.x][tile_pos.y].is_some() {
            tool_state.removing = true;
        }
        tool_state.touched.clear();
    } else if buttons.just_released(MouseButton::Left) {
        tool_state.active = false;
        tool_state.removing = false;
        tool_state.touched.clear();
    }
}

fn designate(
    selected_tool: Res<SelectedTool>,
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    cursor_snapped: Res<CursorSnapped>,
    mut designations: ResMut<Designations>,
    mut tool_state: ResMut<DesignationToolState>,
    tilemap_handle: Res<TilemapHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    if !tool_state.active {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) && !cursor_snapped.is_changed() {
        return;
    }

    let Some(tile_pos) = cursor_snapped.tile_pos else {
        return;
    };

    if tool_state.touched.contains(&tile_pos) {
        return;
    };

    let Some(world_pos_snapped) = cursor_snapped.world_pos else {
        return;
    };

    if tool_state.removing {
        if let Some(designation) = designations.0[tile_pos.x][tile_pos.y].take() {
            commands.entity(designation.indicator).despawn();
        }
        return;
    }

    let Some(tilemap) = tilemaps.get(&tilemap_handle.0) else {
        return;
    };

    let ok = match (selected_tool.0, &tilemap.tiles[tile_pos.x][tile_pos.y]) {
        (Tool::Dig, TileKind::Stone) => true,
        _ => false,
    };

    if !ok {
        // TODO sound
        return;
    }

    let id = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(crate::tilemap::SCALE * crate::tilemap::TILE_SIZE),
                    color: Color::AQUAMARINE.with_a(0.5),
                    ..default()
                },
                transform: Transform::from_translation(world_pos_snapped.extend(1.)),
                ..default()
            },
            DesignationMarker,
        ))
        .id();

    designations.0[tile_pos.x][tile_pos.y] = Some(Designation {
        kind: DesignationKind::Dig,
        indicator: id,
    });

    tool_state.touched.insert(tile_pos);
}
