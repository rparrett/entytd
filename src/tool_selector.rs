use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    common_assets::CommonAssets,
    radio_button::{RadioButton, RadioButtonGroup, RadioButtonGroupRelation},
    GameState,
};

pub struct ToolSelectorPlugin;
impl Plugin for ToolSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedTool>()
            .add_systems(OnEnter(GameState::Playing), init)
            .add_systems(Update, (update_style, select_tool));
    }
}

#[derive(Component)]
struct ToolButton;

#[derive(Component, Default, Clone, Copy)]
pub enum Tool {
    #[default]
    Dig,
    BuildTower,
    Dance,
}

#[derive(Resource, Default)]
pub struct SelectedTool(pub Tool);

fn init(mut commands: Commands, common: Res<CommonAssets>) {
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
                row_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 1..5 {
                let mut button_command = parent.spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        ..default()
                    },
                    NineSliceTexture::from_image(common.ui_nine_slice.clone()),
                    RadioButton { selected: i == 1 },
                    ToolButton,
                ));

                button_command.with_children(|parent| {
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
                            margin: UiRect::bottom(Val::Px(8.)),
                            ..default()
                        }),
                    );
                });

                if i == 1 {
                    button_command.insert(Tool::Dig);
                }
                if i == 2 {
                    button_command.insert(Tool::BuildTower);
                }

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
