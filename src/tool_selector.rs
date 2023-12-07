use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    common_assets::CommonAssets,
    radio_button::{RadioButton, RadioButtonGroup, RadioButtonGroupRelation},
    tilemap::{AtlasHandle, SCALE, TILE_SIZE},
    GameState,
};

pub struct ToolSelectorPlugin;
impl Plugin for ToolSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTool>()
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(Update, (update_style, select_tool, keyboard));
    }
}

#[derive(Component)]
struct ToolButton;

#[derive(Component)]
struct ToolPortrait;

#[derive(Component, Default, Clone, Copy)]
pub enum Tool {
    #[default]
    Dig,
    BuildTower,
    Dance,
}
impl Tool {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Dig => 103 * 31 + 1,
            Self::BuildTower => 103 * 21 + 31,
            Self::Dance => 103 * 31 + 17,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            Self::Dig => 1,
            Self::BuildTower => 2,
            Self::Dance => 3,
        }
    }
    pub fn from_index(val: usize) -> Self {
        match val {
            1 => Tool::Dig,
            2 => Tool::BuildTower,
            3 => Tool::Dance,
            _ => Tool::Dance,
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedTool(pub Tool);

fn init(mut commands: Commands, common: Res<CommonAssets>, atlas_handle: Res<AtlasHandle>) {
    let mut tool_button_ids = vec![];

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.),
                right: Val::Px(5.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(5.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 1..4 {
                let kind = Tool::from_index(i);

                let mut button_command = parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    NineSliceTexture::from_image(common.ui_nine_slice.clone()),
                    RadioButton { selected: i == 1 },
                    ToolButton,
                    kind,
                ));

                button_command.with_children(|parent| {
                    parent.spawn((
                        AtlasImageBundle {
                            style: Style {
                                width: Val::Px(TILE_SIZE.x * SCALE.x),
                                height: Val::Px(TILE_SIZE.y * SCALE.y),
                                ..default()
                            },
                            texture_atlas: atlas_handle.0.clone(),
                            texture_atlas_image: UiTextureAtlasImage {
                                index: kind.atlas_index(),
                                ..default()
                            },
                            ..default()
                        },
                        ToolPortrait,
                    ));
                    parent.spawn(
                        TextBundle::from_section(
                            format!("{}", i),
                            TextStyle {
                                font_size: 18.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::top(Val::Px(4.)),
                            ..default()
                        }),
                    );
                });

                let entity = button_command.id();

                tool_button_ids.push(entity);
            }
        });

    let tool_group_id = commands
        .spawn(RadioButtonGroup {
            entities: tool_button_ids.clone(),
        })
        .id();

    for id in tool_button_ids.iter() {
        commands
            .entity(*id)
            .insert(RadioButtonGroupRelation(tool_group_id));
    }
}

fn update_style(
    mut commands: Commands,
    mut query: Query<(Entity, &RadioButton), (Changed<RadioButton>, With<ToolButton>)>,
    common: Res<CommonAssets>,
) {
    for (entity, radio) in query.iter_mut() {
        if radio.selected {
            commands.entity(entity).insert(NineSliceTexture::from_image(
                common.ui_nine_slice_selected.clone(),
            ));
        } else {
            commands
                .entity(entity)
                .insert(NineSliceTexture::from_image(common.ui_nine_slice.clone()));
        }
    }
}

fn select_tool(
    mut query: Query<(&RadioButton, &Tool), (Changed<RadioButton>, With<ToolButton>)>,
    mut selected_tool: ResMut<SelectedTool>,
) {
    for (radio, tool) in query.iter_mut() {
        if radio.selected {
            selected_tool.0 = *tool;
        }
    }
}

fn keyboard(
    mut query: Query<(&mut RadioButton, &Tool), With<ToolButton>>,
    keys: Res<Input<KeyCode>>,
) {
    let index = if keys.just_pressed(KeyCode::Key1) {
        1
    } else if keys.just_pressed(KeyCode::Key2) {
        2
    } else if keys.just_pressed(KeyCode::Key3) {
        3
    } else if keys.just_pressed(KeyCode::Key4) {
        4
    } else {
        return;
    };

    for (mut radio, tool) in &mut query {
        if index == tool.index() {
            radio.selected = true;
        }
    }
}
