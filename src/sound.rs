use bevy::{audio::Volume, prelude::*};

use crate::{settings::MusicSetting, GameState};

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .add_systems(OnExit(GameState::Loading), start_music);
    }
}

#[derive(Resource)]
pub struct SoundAssets {
    pub bgm: Handle<AudioSource>,
    pub pickaxe: Handle<AudioSource>,
    pub wave: Handle<AudioSource>,
}
impl FromWorld for SoundAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let bgm = asset_server.load("bgm-hzsmith.ogg");
        let pickaxe = asset_server.load("pickaxe.ogg");
        let wave = asset_server.load("wave.ogg");
        SoundAssets { bgm, pickaxe, wave }
    }
}

#[derive(Component)]
pub struct MusicController;

fn start_music(
    mut commands: Commands,
    music_setting: Res<MusicSetting>,
    audio_assets: Res<SoundAssets>,
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
