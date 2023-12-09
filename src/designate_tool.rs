use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    currency::Currency,
    cursor::CursorSnapped,
    layer,
    tilemap::{AtlasHandle, TileKind, TilePos, Tilemap},
    tool_selector::{SelectedTool, Tool},
    GameState,
};

const DESIGNATE_DIG_OK: Color = Color::rgba(0., 1.0, 1.0, 0.2);
const DESIGNATE_DANCE_OK: Color = Color::rgba(1.0, 0.0, 1.0, 0.2);
const DESIGNATE_NOT_OK: Color = Color::rgba(1.0, 0.0, 0.0, 0.8);

pub struct DesignateToolPlugin;
impl Plugin for DesignateToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DesignationToolState>();
        app.init_resource::<Designations>();
        app.add_systems(
            Update,
            (update_cursor, show_cursor).run_if(in_state(GameState::Playing)),
        );
        app.add_systems(
            Update,
            (update_tool_state, designate)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
        app.add_systems(OnEnter(GameState::Playing), init_cursor);
        app.add_systems(OnExit(GameState::GameOver), cleanup);
    }
}

#[derive(Component)]
struct DesignateToolCursor;

#[derive(Copy, Clone, Debug)]
pub enum DesignationKind {
    Dig,
    BuildTower,
    Dance,
}
impl From<Tool> for DesignationKind {
    fn from(value: Tool) -> Self {
        match value {
            Tool::BuildTower => DesignationKind::BuildTower,
            Tool::Dig => DesignationKind::Dig,
            Tool::Dance => DesignationKind::Dance,
        }
    }
}
impl DesignationKind {
    fn ok_color(&self) -> Color {
        match self {
            DesignationKind::Dig => DESIGNATE_DIG_OK,
            DesignationKind::BuildTower => Color::rgb_u8(82, 94, 173),
            DesignationKind::Dance => DESIGNATE_DANCE_OK,
        }
    }
    fn ok_atlas_index(&self) -> usize {
        match self {
            DesignationKind::Dig => TileKind::WhitePickaxe.atlas_index(),
            DesignationKind::BuildTower => TileKind::TowerBlueprint.atlas_index(),
            DesignationKind::Dance => TileKind::White.atlas_index(),
        }
    }
    fn not_ok_color(&self) -> Color {
        match self {
            DesignationKind::Dig | DesignationKind::BuildTower | DesignationKind::Dance => {
                DESIGNATE_NOT_OK
            }
        }
    }
    fn not_ok_atlas_index(&self) -> usize {
        match self {
            DesignationKind::Dig | DesignationKind::BuildTower | DesignationKind::Dance => {
                TileKind::WhiteCircleNo.atlas_index()
            }
        }
    }
    pub fn price(&self) -> Currency {
        match self {
            DesignationKind::BuildTower => Currency {
                metal: 1,
                stone: 10,
                crystal: 0,
            },
            _ => Currency::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Designation {
    pub kind: DesignationKind,
    pub indicator: Entity,
    pub workers: u32,
}

#[derive(Component)]
struct DesignationMarker;

#[derive(Resource, Default)]
pub struct Designations(pub HashMap<TilePos, Designation>);

#[derive(Resource, Default)]
struct DesignationToolState {
    active: bool,
    removing: bool,
    touched: HashSet<TilePos>,
}

fn init_cursor(mut commands: Commands, atlas_handle: Res<AtlasHandle>) {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: DesignationKind::Dig.not_ok_atlas_index(),
                color: DesignationKind::Dig.not_ok_color(),
                ..default()
            },
            texture_atlas: atlas_handle.0.clone(),
            visibility: Visibility::Hidden,
            transform: Transform::from_xyz(0., 0., layer::CURSOR)
                .with_scale(crate::tilemap::SCALE.extend(1.)),
            ..default()
        },
        DesignateToolCursor,
    ));
}

fn update_cursor(
    selected_tool: Res<SelectedTool>,
    cursor_snapped: Res<CursorSnapped>,
    mut query: Query<(&mut Transform, &mut TextureAtlasSprite), With<DesignateToolCursor>>,
    tilemap_query: Query<&Tilemap>,
    currency: Res<Currency>,
) {
    if !cursor_snapped.is_changed() && !currency.is_changed() && !selected_tool.is_changed() {
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

        let Ok(tilemap) = tilemap_query.get_single() else {
            return;
        };

        let kind = tilemap.tiles[tile_pos.x][tile_pos.y];

        let designation = DesignationKind::from(selected_tool.0);

        let ok = match selected_tool.0 {
            Tool::Dig if kind.diggable() => true,
            Tool::BuildTower if kind.buildable() => true,
            Tool::Dance if kind.buildable() => true,
            _ => false,
        };

        let has_money = currency.has(&designation.price());

        // TODO separate cursor for the no-money situation

        if ok && has_money {
            sprite.index = designation.ok_atlas_index();
            sprite.color = designation.ok_color();
        } else {
            sprite.index = designation.not_ok_atlas_index();
            sprite.color = designation.not_ok_color();
        }
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
        Tool::Dig | Tool::BuildTower | Tool::Dance => Visibility::Visible,
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
        if designations.0.contains_key(&tile_pos) {
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
    tilemap_query: Query<&Tilemap>,
    atlas_handle: Res<AtlasHandle>,
    mut currency: ResMut<Currency>,
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
        if let Some(designation) = designations.0.remove(&tile_pos) {
            commands.entity(designation.indicator).despawn();
        }

        return;
    }

    if designations.0.contains_key(&tile_pos) {
        return;
    }

    let Ok(tilemap) = tilemap_query.get_single() else {
        return;
    };

    let kind = tilemap.tiles[tile_pos.x][tile_pos.y];

    let ok = match selected_tool.0 {
        Tool::Dig if kind.diggable() => true,
        Tool::BuildTower if kind.buildable() => true,
        Tool::Dance if kind.buildable() => true,
        _ => false,
    };

    if !ok {
        // TODO sound
        return;
    }

    let designation_kind = DesignationKind::from(selected_tool.0);
    if currency.try_sub(&designation_kind.price()).is_err() {
        // TODO sound
        return;
    }

    let id = commands
        .spawn((
            SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: designation_kind.ok_atlas_index(),
                    color: designation_kind.ok_color(),
                    ..default()
                },
                texture_atlas: atlas_handle.0.clone(),
                transform: Transform::from_translation(world_pos_snapped.extend(layer::BLUEPRINT))
                    .with_scale(crate::tilemap::SCALE.extend(1.)),
                ..default()
            },
            DesignationMarker,
            #[cfg(feature = "inspector")]
            Name::new("DesignationMarker"),
        ))
        .id();

    designations.0.insert(
        tile_pos,
        Designation {
            kind: designation_kind,
            indicator: id,
            workers: 0,
        },
    );

    tool_state.touched.insert(tile_pos);
}

fn cleanup(
    mut commands: Commands,
    query: Query<Entity, Or<(With<DesignationMarker>, With<DesignateToolCursor>)>>,
    mut designations: ResMut<Designations>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    designations.0.clear();
}
