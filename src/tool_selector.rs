use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::GameState;

pub struct ToolSelectorPlugin;
impl Plugin for ToolSelectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), init);
    }
}

fn init(mut commands: Commands, server: Res<AssetServer>) {
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
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Px(60.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        ..default()
                    })
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
                    });
            }
        });
}
