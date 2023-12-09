use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    game::Won,
    ui::{UiAssets, BUTTON_TEXT, TITLE_TEXT},
    util::cleanup,
    GameState,
};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), init);
        app.add_systems(OnExit(GameState::GameOver), cleanup::<GameOverScene>);
        app.add_systems(Update, menu_button.run_if(in_state(GameState::GameOver)));
    }
}

#[derive(Component)]
struct GameOverScene;

#[derive(Component)]
struct MenuButton;

fn init(mut commands: Commands, ui_assets: Res<UiAssets>, won: Res<Won>) {
    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(45.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = TextStyle {
        font_size: 18.0,
        color: BUTTON_TEXT,
        ..default()
    };

    let title_text_style = TextStyle {
        font_size: 18.0,
        color: TITLE_TEXT,
        ..default()
    };

    let container = commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            NineSliceTexture::from_image(ui_assets.nine_slice_container.clone()),
            GameOverScene,
        ))
        .id();

    let title = commands
        .spawn(
            TextBundle::from_section(
                if won.0 {
                    "You win!".to_string()
                } else {
                    "You lose!".to_string()
                },
                title_text_style,
            )
            .with_style(Style {
                margin: UiRect {
                    bottom: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .id();

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                ..default()
            },
            NineSliceTexture::from_image(ui_assets.nine_slice.clone()),
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Back to menu",
                button_text_style.clone(),
            ));
        })
        .id();

    commands
        .entity(container)
        .push_children(&[title, play_button]);
}

fn menu_button(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<MenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
}
