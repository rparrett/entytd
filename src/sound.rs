use bevy::{audio::Volume, prelude::*};

use crate::{loading::LoadingAssets, settings::MusicSetting, GameState};

pub struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundAssets>()
            .add_systems(OnExit(GameState::Loading), start_music)
            .add_systems(Update, fade_music.run_if(not(in_state(GameState::Loading))));
    }
}

#[derive(Resource)]
pub struct SoundAssets {
    pub bgm: Handle<AudioSource>,
    pub pickaxe: Handle<AudioSource>,
    pub wave: Handle<AudioSource>,
    pub tutorial: Handle<AudioSource>,
    pub bad: Handle<AudioSource>,
}
impl FromWorld for SoundAssets {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let bgm = asset_server.load("audio/bgm-hzsmith.ogg");
        let pickaxe = asset_server.load("audio/pickaxe.ogg");
        let wave = asset_server.load("audio/wave.ogg");
        let tutorial = asset_server.load("audio/tutorial.ogg");
        let bad = asset_server.load("audio/bad.ogg");

        let mut loading_assets = world.resource_mut::<LoadingAssets>();
        loading_assets.0.push(bgm.id().into());
        loading_assets.0.push(pickaxe.id().into());
        loading_assets.0.push(wave.id().into());
        loading_assets.0.push(tutorial.id().into());
        loading_assets.0.push(bad.id().into());

        SoundAssets {
            bgm,
            pickaxe,
            wave,
            tutorial,
            bad,
        }
    }
}

#[derive(Component)]
pub struct MusicController;

/// Music fades in over this amount of seconds.
#[derive(Component)]
pub struct MusicFade {
    seconds: f32,
    remaining: f32,
}
impl Default for MusicFade {
    fn default() -> Self {
        let seconds = 4.;
        let remaining = seconds;
        Self { seconds, remaining }
    }
}

fn start_music(
    mut commands: Commands,
    music_setting: Res<MusicSetting>,
    audio_assets: Res<SoundAssets>,
) {
    let initial_volume = if **music_setting == 0 {
        0.0
    } else {
        f32::EPSILON
    };

    commands.spawn((
        AudioBundle {
            source: audio_assets.bgm.clone(),
            settings: PlaybackSettings::LOOP.with_volume(Volume::new(initial_volume)),
        },
        MusicController,
        MusicFade::default(),
    ));
}

fn fade_music(
    mut commands: Commands,
    mut query: Query<(Entity, &AudioSink, &mut MusicFade), With<MusicController>>,
    music_setting: Res<MusicSetting>,
    time: Res<Time>,
) {
    if let Ok((entity, sink, mut fade)) = query.get_single_mut() {
        fade.remaining -= time.delta_seconds();

        let progress = (1.0 - fade.remaining / fade.seconds).min(1.);

        sink.set_volume(progress * **music_setting as f32 / 100.);

        if progress >= 1.0 {
            commands.entity(entity).remove::<MusicFade>();
        }
    }
}
