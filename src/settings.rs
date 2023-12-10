use std::fmt::Display;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct MusicSetting(u8);
impl Default for MusicSetting {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Resource, Deref, DerefMut, Debug, Serialize, Deserialize, Clone)]
pub struct SfxSetting(u8);
impl Default for SfxSetting {
    fn default() -> Self {
        Self(100)
    }
}
#[derive(Resource, Debug, Default, Serialize, Deserialize, Clone)]
pub enum DifficultySetting {
    #[default]
    Normal,
    Hard,
    Impossible,
}
impl DifficultySetting {
    pub fn next(&self) -> Self {
        match self {
            Self::Normal => Self::Hard,
            Self::Hard => Self::Impossible,
            Self::Impossible => Self::Normal,
        }
    }
}
impl Display for DifficultySetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Normal => "Normal",
                Self::Hard => "Hard",
                Self::Impossible => "Impossible",
            }
        )
    }
}
#[derive(Resource, Default, Serialize, Deserialize, Debug, Clone)]
pub enum ParticlesSetting {
    #[default]
    Lots,
    Low,
    None,
}

impl ParticlesSetting {
    pub fn next(&self) -> Self {
        match self {
            Self::Low => Self::Lots,
            Self::Lots => Self::None,
            Self::None => Self::Low,
        }
    }
    pub const fn hit_amt(&self) -> usize {
        match self {
            Self::Lots => 12,
            Self::Low => 6,
            Self::None => 0,
        }
    }
    pub const fn kill_amt(&self) -> usize {
        match self {
            Self::Lots => 40,
            Self::Low => 20,
            Self::None => 0,
        }
    }
}
impl Display for ParticlesSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Lots => "Lots of particles",
                Self::Low => "Some particles",
                Self::None => "No particles",
            }
        )
    }
}
