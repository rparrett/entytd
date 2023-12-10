use crate::settings::{DifficultySetting, MusicSetting, ParticlesSetting, SfxSetting};

use bevy::prelude::*;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
const SAVE_FILE: &str = "save.ron";
#[cfg(target_arch = "wasm32")]
const LOCAL_STORAGE_KEY: &str = "undefended-save";

pub struct SavePlugin;
impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, save_system);
        app.add_systems(Startup, load_system);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SaveFile {
    sfx: SfxSetting,
    music: MusicSetting,
    particles: ParticlesSetting,
    difficulty: DifficultySetting,
}

pub fn load_system(mut commands: Commands) {
    commands.insert_resource(SfxSetting::default());
    commands.insert_resource(MusicSetting::default());
    commands.insert_resource(ParticlesSetting::default());
    commands.insert_resource(DifficultySetting::default());

    #[cfg(not(target_arch = "wasm32"))]
    {
        let Ok(file) = std::fs::File::open(SAVE_FILE) else {
            return;
        };

        let save_file: SaveFile = match ron::de::from_reader(file) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to deserialize save file: {:?}", e);
                return;
            }
        };

        info!("Loaded settings: {:?}", save_file);

        commands.insert_resource(save_file.sfx);
        commands.insert_resource(save_file.music);
        commands.insert_resource(save_file.particles);
        commands.insert_resource(save_file.difficulty);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let Some(window) = web_sys::window() else {
            return;
        };

        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };

        let Ok(Some(item)) = storage.get_item(LOCAL_STORAGE_KEY) else {
            return;
        };

        let save_file: SaveFile = match ron::de::from_str(&item) {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to serialize save file: {:?}", e);
                return;
            }
        };

        info!("Loaded settings: {:?}", save_file);

        commands.insert_resource(save_file.sfx);
        commands.insert_resource(save_file.music);
        commands.insert_resource(save_file.particles);
        commands.insert_resource(save_file.difficulty);
    }
}

pub fn save_system(
    sfx: Res<SfxSetting>,
    music: Res<MusicSetting>,
    difficulty: Res<DifficultySetting>,
    particles: Res<ParticlesSetting>,
) {
    let sfx_changed = sfx.is_changed() && !sfx.is_added();
    let music_changed = music.is_changed() && !music.is_added();
    let difficulty_changed = difficulty.is_changed() && !difficulty.is_added();
    let particles_changed = particles.is_changed() && !particles.is_added();

    if !sfx_changed && !music_changed && !difficulty_changed && !particles_changed {
        return;
    }

    info!("Saving settings.");

    let save_file = SaveFile {
        sfx: sfx.clone(),
        music: music.clone(),
        difficulty: difficulty.clone(),
        particles: particles.clone(),
    };

    let pretty = PrettyConfig::new();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = match std::fs::File::create(SAVE_FILE) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to create save file: {:?}", e);
                return;
            }
        };

        if let Err(e) = ron::ser::to_writer_pretty(file, &save_file, pretty) {
            warn!("Failed to serialize save data: {:?}", e);
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let data = match ron::ser::to_string_pretty(&save_file, pretty) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to serialize save data: {:?}", e);
                return;
            }
        };

        let Some(window) = web_sys::window() else {
            return;
        };

        let Ok(Some(storage)) = window.local_storage() else {
            return;
        };

        if let Err(e) = storage.set_item(LOCAL_STORAGE_KEY, data.as_str()) {
            warn!("Failed to store save file: {:?}", e);
        }
    }
}
