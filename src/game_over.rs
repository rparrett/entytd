use bevy::prelude::*;

use crate::{
    game::Won,
    settings::DifficultySetting,
    stats::Stats,
    ui::{slice_image_mode, UiAssets, BUTTON_TEXT, TITLE_TEXT},
    GameState,
};

pub struct GameOverPlugin;
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), init);
        app.add_systems(Update, menu_button.run_if(in_state(GameState::GameOver)));
    }
}

#[derive(Component)]
struct MenuButton;

fn init(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    won: Res<Won>,
    stats: Res<Stats>,
    difficulty: Res<DifficultySetting>,
) {
    let button_style = Node {
        width: Val::Px(250.0),
        height: Val::Px(45.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_text_style = (
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(BUTTON_TEXT),
    );

    let title_text_style = (
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(TITLE_TEXT),
    );

    let container = commands
        .spawn((
            Node {
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                row_gap: Val::Px(10.),
                ..default()
            },
            ImageNode {
                image: ui_assets.nine_panel.clone(),
                image_mode: slice_image_mode(),
                ..default()
            },
            DespawnOnExit(GameState::GameOver),
        ))
        .id();

    let title = commands
        .spawn((
            Text::new(if won.0 {
                "You win!".to_string()
            } else {
                "You lose!".to_string()
            }),
            title_text_style.clone(),
        ))
        .id();

    let stats_container = commands
        .spawn(Node {
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::flex(1.0), GridTrack::min_content()],
            row_gap: Val::Px(5.),
            column_gap: Val::Px(40.),
            ..default()
        })
        .id();

    let kills_label = commands
        .spawn((Text::new("Enemies Defeated"), title_text_style.clone()))
        .id();
    let kills = commands
        .spawn((
            Text::new(format!("{}", stats.kills)),
            title_text_style.clone(),
        ))
        .id();

    let mined_label = commands
        .spawn((Text::new("Resources Mined"), title_text_style.clone()))
        .id();
    let mined = commands
        .spawn((
            Text::new(format!("{}", stats.mined)),
            title_text_style.clone(),
        ))
        .id();

    let built_label = commands
        .spawn((Text::new("Towers Built"), title_text_style.clone()))
        .id();
    let built = commands
        .spawn((
            Text::new(format!("{}", stats.towers)),
            title_text_style.clone(),
        ))
        .id();

    let difficulty_label = commands
        .spawn((Text::new("Difficulty"), title_text_style.clone()))
        .id();
    let difficulty = commands
        .spawn((
            Text::new(format!("{}", *difficulty)),
            title_text_style.clone(),
        ))
        .id();

    let play_button = commands
        .spawn((
            Button,
            button_style.clone(),
            ImageNode {
                image: ui_assets.nine_button.clone(),
                image_mode: slice_image_mode(),
                ..default()
            },
            MenuButton,
        ))
        .with_children(|parent| {
            parent.spawn((Text::new("Back to menu"), button_text_style.clone()));
        })
        .id();

    commands.entity(stats_container).add_children(&[
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
        .add_children(&[title, stats_container, play_button]);
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
