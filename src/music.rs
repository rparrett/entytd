use bevy::{audio::Volume, prelude::*};

use crate::{common_assets::Sounds, settings::MusicSetting, GameState};

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), start_music);
    }
}

#[derive(Component)]
pub struct MusicController;

fn start_music(
    mut commands: Commands,
    music_setting: Res<MusicSetting>,
    audio_assets: Res<Sounds>,
) {
    commands.spawn((
        AudioBundle {
            source: audio_assets.bgm.clone(),
            settings: PlaybackSettings::LOOP
                .with_volume(Volume::new_absolute(**music_setting as f32 / 100.)),
        },
        MusicController,
    ));
}
