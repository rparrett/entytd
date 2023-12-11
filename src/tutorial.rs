use bevy::{audio::Volume, prelude::*};
use bevy_nine_slice_ui::NineSliceTexture;

use crate::{
    currency::Currency,
    designate_tool::{DesignationKind, Designations},
    settings::{SfxSetting, TutorialFinishedSetting},
    sound::SoundAssets,
    spawner::SpawningPaused,
    ui::UiAssets,
    util::cleanup,
    GameState,
};

pub struct TutorialPlugin;
impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(
                OnEnter(GameState::Playing),
                init_tutorial.run_if(tutorial_not_finished),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (reset, cleanup::<TutorialScene>),
            )
            .add_systems(
                Update,
                (camera, dug, dug_more, built, update)
                    .run_if(in_state(GameState::Playing).and_then(tutorial_not_finished)),
            );
    }
}

#[derive(Component)]
pub struct TutorialScene;

#[derive(Component)]
pub struct TutorialText;

#[derive(Resource, Default)]
pub enum TutorialState {
    #[default]
    CameraOne,
    CameraTwo,
    Dig,
    DigMore,
    Build,
    Done,
}

fn tutorial_not_finished(setting: Res<TutorialFinishedSetting>) -> bool {
    return !setting.0;
}

pub fn init_tutorial(mut commands: Commands, ui_assets: Res<UiAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Px(100.),
                        ..default()
                    },
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    padding: UiRect::all(Val::Px(20.)),
                    max_width: Val::Px(500.),
                    width: Val::Px(500.),
                    ..default()
                },
                ..default()
            },
            NineSliceTexture::from_image(ui_assets.nine_slice.clone()),
            Name::new("TutorialContainer"),
            TutorialScene,
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle::from_section(
                "This is your new peaceful mountain home. Use the WASD, QZSD, or arrow keys to look around. Hold LSHIFT or RSHIFT to while moving to move the camera faster."
                    .to_string(),
                TextStyle {
                    font_size: 18.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ), TutorialText));
        });
}

pub fn camera(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    mut tutorial_state: ResMut<TutorialState>,
    sfx_setting: Res<SfxSetting>,
    sound_assets: Res<SoundAssets>,
) {
    let moving = keys.any_pressed([
        KeyCode::Right,
        KeyCode::D,
        KeyCode::Left,
        KeyCode::A,
        KeyCode::Q,
        KeyCode::Up,
        KeyCode::W,
        KeyCode::Z,
        KeyCode::Down,
        KeyCode::S,
    ]);

    let fast = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

    match *tutorial_state {
        TutorialState::CameraOne if moving => {
            *tutorial_state = TutorialState::CameraTwo;
        }
        TutorialState::CameraTwo if (moving && fast) => {
            *tutorial_state = TutorialState::Dig;

            commands.spawn(AudioBundle {
                source: sound_assets.tutorial.clone(),
                settings: PlaybackSettings::DESPAWN
                    .with_volume(Volume::new_absolute(**sfx_setting as f32 / 100.)),
            });
        }
        _ => {}
    };
}

pub fn dug(spawning_paused: Res<SpawningPaused>, mut tutorial_state: ResMut<TutorialState>) {
    match *tutorial_state {
        // Player may have skipped the camera tutorial and just started digging.
        TutorialState::CameraOne | TutorialState::CameraTwo | TutorialState::Dig
            if !spawning_paused.0 =>
        {
            *tutorial_state = TutorialState::DigMore;

            // No tutorial sound here because it would clash with the wave start sound.
        }
        _ => {}
    }
}

pub fn dug_more(
    mut commands: Commands,
    currency: Res<Currency>,
    mut tutorial_state: ResMut<TutorialState>,
    sfx_setting: Res<SfxSetting>,
    sound_assets: Res<SoundAssets>,
) {
    if matches!(*tutorial_state, TutorialState::DigMore)
        && currency.has(&DesignationKind::BuildTower.price())
    {
        *tutorial_state = TutorialState::Build;

        commands.spawn(AudioBundle {
            source: sound_assets.tutorial.clone(),
            settings: PlaybackSettings::DESPAWN
                .with_volume(Volume::new_absolute(**sfx_setting as f32 / 100.)),
        });
    }
}

pub fn built(designations: Res<Designations>, mut tutorial_state: ResMut<TutorialState>) {
    let built = designations
        .0
        .iter()
        .any(|(_, v)| matches!(v.kind, DesignationKind::BuildTower));
    if built {
        *tutorial_state = TutorialState::Done;
    }
}

fn update(
    mut commands: Commands,
    tutorial_state: Res<TutorialState>,
    mut query: Query<&mut Text, With<TutorialText>>,
    scene_query: Query<Entity, With<TutorialScene>>,
    mut setting: ResMut<TutorialFinishedSetting>,
) {
    if matches!(*tutorial_state, TutorialState::Done) {
        for entity in &scene_query {
            commands.entity(entity).despawn_recursive()
        }
        setting.0 = true;
        return;
    }

    for mut text in &mut query {
        match *tutorial_state {
            TutorialState::Dig => {
                text.sections[0].value = "You can use the number keys or mouse to select a tool on the right. With dig tool (1), click and hold the left mouse button to paint the stone you want your workers to excavate.".to_string();
            }
            TutorialState::DigMore => {
                text.sections[0].value =
                    "Uh Oh! Some entities approach, and they look unfriendly! We'll need 15 stone and 1 metal for our defenses. Make sure all your workers are working!".to_string();
            }
            TutorialState::Build => {
                text.sections[0].value =
                    "Use the build tool to select a nice spot near the main road to build a tower."
                        .to_string();
            }
            _ => {}
        }
    }
}

fn reset(mut commands: Commands) {
    commands.insert_resource(TutorialState::default());
}
