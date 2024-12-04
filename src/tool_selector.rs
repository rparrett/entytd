use bevy::prelude::*;

use crate::{
    radio_button::{RadioButton, RadioButtonGroup, RadioButtonGroupRelation},
    tilemap::{AtlasHandle, TileKind, SCALE, TILE_SIZE},
    ui::{UiAssets, BUTTON_TEXT},
    util::cleanup,
    GameState,
};

pub struct ToolSelectorPlugin;
impl Plugin for ToolSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTool>()
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(Update, (update_style, select_tool, keyboard))
            .add_systems(OnExit(GameState::GameOver), cleanup::<ToolContainer>);
    }
}

#[derive(Component)]
struct ToolContainer;

#[derive(Component)]
struct ToolButton;

#[derive(Component)]
struct ToolPortrait;

#[derive(Component, Default, Debug, Clone, Copy)]
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
            Self::BuildTower => TileKind::Tower.atlas_index(),
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
            _ => Tool::Dance,
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectedTool(pub Tool);

fn init(mut commands: Commands, ui_assets: Res<UiAssets>, atlas_handle: Res<AtlasHandle>) {
    let mut tool_button_ids = vec![];

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                height: Val::Percent(100.),
                right: Val::Px(5.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(5.),
                ..default()
            },
            ToolContainer,
        ))
        .with_children(|parent| {
            for i in 1..3 {
                let kind = Tool::from_index(i);

                let mut button_command = parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(64.0),
                        height: Val::Px(64.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ImageNode {
                        image: ui_assets.nine_button.clone(),
                        ..default()
                    },
                    // TODO 9 slice
                    RadioButton { selected: i == 1 },
                    ToolButton,
                    kind,
                ));

                button_command.with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(TILE_SIZE.x * SCALE.x),
                            height: Val::Px(TILE_SIZE.y * SCALE.y),
                            ..default()
                        },
                        ImageNode {
                            image: atlas_handle.image.clone().into(),
                            texture_atlas: Some(TextureAtlas {
                                layout: atlas_handle.layout.clone(),
                                index: kind.atlas_index(),
                            }),
                            ..default()
                        },
                        ToolPortrait,
                    ));
                    parent.spawn((
                        Text::new(format!("{}", i)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(BUTTON_TEXT),
                        Node {
                            margin: UiRect::top(Val::Px(6.)),
                            ..default()
                        },
                    ));
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
    mut query: Query<(&RadioButton, &mut ImageNode), (Changed<RadioButton>, With<ToolButton>)>,
    ui_assets: Res<UiAssets>,
) {
    for (radio, mut image_node) in query.iter_mut() {
        if radio.selected {
            image_node.image = ui_assets.nine_button_selected.clone();
        } else {
            image_node.image = ui_assets.nine_button.clone();
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
    keys: Res<ButtonInput<KeyCode>>,
) {
    let index = if keys.just_pressed(KeyCode::Digit1) {
        1
    } else if keys.just_pressed(KeyCode::Digit2) {
        2
    } else if keys.just_pressed(KeyCode::Digit3) {
        3
    } else if keys.just_pressed(KeyCode::Digit4) {
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
