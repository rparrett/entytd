use bevy::prelude::*;
use bevy_nine_slice_ui::NineSliceUiTexture;

use crate::{
    game::Won,
    settings::DifficultySetting,
    stats::Stats,
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

fn init(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    won: Res<Won>,
    stats: Res<Stats>,
    difficulty: Res<DifficultySetting>,
) {
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
                    row_gap: Val::Px(10.),
                    ..default()
                },
                ..default()
            },
            NineSliceUiTexture::from_image(ui_assets.nine_panel.clone()),
            GameOverScene,
        ))
        .id();

    let title = commands
        .spawn(TextBundle::from_section(
            if won.0 {
                "You win!".to_string()
            } else {
                "You lose!".to_string()
            },
            title_text_style.clone(),
        ))
        .id();

    let stats_container = commands
        .spawn(NodeBundle {
            style: Style {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::min_content()],
                row_gap: Val::Px(5.),
                column_gap: Val::Px(40.),
                ..default()
            },
            ..default()
        })
        .id();

    let kills_label = commands
        .spawn(TextBundle::from_section(
            "Enemies Defeated",
            title_text_style.clone(),
        ))
        .id();
    let kills = commands
        .spawn(TextBundle::from_section(
            format!("{}", stats.kills),
            title_text_style.clone(),
        ))
        .id();

    let mined_label = commands
        .spawn(TextBundle::from_section(
            "Resources Mined",
            title_text_style.clone(),
        ))
        .id();
    let mined = commands
        .spawn(TextBundle::from_section(
            format!("{}", stats.mined),
            title_text_style.clone(),
        ))
        .id();

    let built_label = commands
        .spawn(TextBundle::from_section(
            "Towers Built",
            title_text_style.clone(),
        ))
        .id();
    let built = commands
        .spawn(TextBundle::from_section(
            format!("{}", stats.towers),
            title_text_style.clone(),
        ))
        .id();

    let difficulty_label = commands
        .spawn(TextBundle::from_section(
            "Difficulty",
            title_text_style.clone(),
        ))
        .id();
    let difficulty = commands
        .spawn(TextBundle::from_section(
            format!("{}", *difficulty),
            title_text_style.clone(),
        ))
        .id();

    let play_button = commands
        .spawn((
            ButtonBundle {
                style: button_style.clone(),
                ..default()
            },
            NineSliceUiTexture::from_image(ui_assets.nine_button.clone()),
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Back to menu",
                button_text_style.clone(),
            ));
        })
        .id();

    commands.entity(stats_container).push_children(&[
        difficulty_label,
        difficulty,
        kills_label,
        kills,
        mined_label,
        mined,
        built_label,
        built,
    ]);

    commands
        .entity(container)
        .push_children(&[title, stats_container, play_button]);
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
