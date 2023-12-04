use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    radio_button::{RadioButton, RadioButtonGroup, RadioButtonGroupRelation},
    GameState,
};

pub struct ToolSelectorPlugin;
impl Plugin for ToolSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init)
            .add_systems(Update, update_style);
    }
}

#[derive(Component)]
struct ToolButton;

fn init(mut commands: Commands, server: Res<AssetServer>) {
    let mut tool_button_ids = vec![];

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                bottom: Val::Px(5.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 1..5 {
                let entity = parent
                    .spawn((
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
                        RadioButton { selected: i == 1 },
                        ToolButton,
                    ))
                    .insert(NineSliceTexture::from_image(
                        server.load("ui_nine_slice.png"),
                    ))
                    .with_children(|parent| {
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
                    })
                    .id();

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
    server: Res<AssetServer>,
) {
    for (entity, radio) in query.iter_mut() {
        if radio.selected {
            commands.entity(entity).insert(NineSliceTexture::from_image(
                server.load("ui_nine_slice_selected.png"),
            ));
        } else {
            commands.entity(entity).insert(NineSliceTexture::from_image(
                server.load("ui_nine_slice.png"),
            ));
        }
    }
}
