use bevy::prelude::*;

use crate::{
    designate_tool::Designations,
    hit_points::HitPoints,
    tilemap::{TileKind, TilePos, Tilemap},
    GameState,
};

pub struct StonePlugin;
impl Plugin for StonePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HitStoneEvent>()
            .add_systems(Update, process_events.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Stone;

#[derive(Debug)]
pub enum StoneHealth {
    Full,
    Hurt,
    Dying,
    Dead,
}
impl From<&HitPoints> for StoneHealth {
    fn from(value: &HitPoints) -> Self {
        if value.is_zero() {
            return Self::Dead;
        }
        let percent = value.fraction();
        if percent <= 0.25 {
            return Self::Dying;
        } else if percent < 1.0 {
            return Self::Hurt;
        } else {
            return Self::Full;
        }
    }
}

#[derive(Event)]
pub struct HitStoneEvent {
    pub entity: Entity,
    pub damage: u32,
}

fn process_events(
    mut commands: Commands,
    mut reader: EventReader<HitStoneEvent>,
    //mut writer: EventWriter<HitStoneEvent>,
    mut query: Query<(
        &mut HitPoints,
        &TilePos,
        &mut TileKind,
        &mut TextureAtlasSprite,
    )>,
    mut designations: ResMut<Designations>,
    mut tilemap_query: Query<&mut Tilemap>,
) {
    for event in reader.read() {
        let Ok(mut map) = tilemap_query.get_single_mut() else {
            return;
        };

        let Ok((mut hp, pos, mut kind, mut sprite)) = query.get_mut(event.entity) else {
            continue;
        };

        if hp.is_zero() {
            continue;
        }

        // TODO sound if dmg > 1

        hp.sub(event.damage);

        let health = StoneHealth::from(&*hp);

        let crystal = matches!(
            *kind,
            TileKind::CrystalHidden
                | TileKind::Crystal
                | TileKind::CrystalHurt
                | TileKind::CrystalDying
        );
        let metal = matches!(
            *kind,
            TileKind::CrystalHidden
                | TileKind::Crystal
                | TileKind::CrystalHurt
                | TileKind::CrystalDying
        );

        if crystal {
            *kind = match health {
                StoneHealth::Dead => TileKind::Dirt,
                StoneHealth::Hurt => TileKind::CrystalHurt,
                StoneHealth::Dying => TileKind::CrystalDying,
                StoneHealth::Full => TileKind::Crystal,
            }
        } else if metal {
            *kind = match health {
                StoneHealth::Dead => TileKind::Dirt,
                StoneHealth::Hurt => TileKind::MetalHurt,
                StoneHealth::Dying => TileKind::MetalDying,
                StoneHealth::Full => TileKind::Metal,
            }
        } else {
            *kind = match health {
                StoneHealth::Dead => TileKind::Dirt,
                StoneHealth::Hurt => TileKind::StoneHurt,
                StoneHealth::Dying => TileKind::StoneDying,
                StoneHealth::Full => TileKind::Stone,
            }
        }

        sprite.index = kind.atlas_index();
        map.tiles[pos.x][pos.y] = *kind;

        if hp.is_zero() {
            if let Some(designation) = designations.0.remove(pos) {
                commands.entity(designation.indicator).despawn();
            }
        }

        // TODO give resources
    }
}
