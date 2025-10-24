use bevy::prelude::*;

use crate::{
    currency::Currency,
    designate_tool::Designations,
    hit_points::HitPoints,
    particle::ParticleKind,
    settings::ParticlesSetting,
    spawner::SpawningPaused,
    stats::Stats,
    tilemap::{Map, TileEntities, TileKind, TilePos},
    GameState,
};

pub struct StonePlugin;
impl Plugin for StonePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HitStoneMessage>()
            .add_message::<RevealStoneMessage>()
            .add_systems(
                Update,
                (hit_events, reveal_events).run_if(in_state(GameState::Playing)),
            );
    }
}

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
        let fraction = value.fraction();
        if fraction <= 0.25 {
            return Self::Dying;
        } else if fraction < 1.0 {
            return Self::Hurt;
        }

        Self::Full
    }
}

#[derive(Message)]
pub struct HitStoneMessage {
    pub entity: Entity,
    pub damage: u32,
}

#[derive(Message)]
pub struct RevealStoneMessage(Entity);

fn hit_events(
    mut commands: Commands,
    mut hit_reader: MessageReader<HitStoneMessage>,
    mut reveal_writer: MessageWriter<RevealStoneMessage>,
    mut query: Query<(&mut HitPoints, &TilePos, &mut TileKind, &mut Sprite)>,
    mut designations: ResMut<Designations>,
    mut tilemap_query: Query<(&mut Map, &TileEntities)>,
    mut currency: ResMut<Currency>,
    particle_settings: Res<ParticlesSetting>,
    mut spawning_paused: ResMut<SpawningPaused>,
    mut stats: ResMut<Stats>,
) {
    for event in hit_reader.read() {
        let Ok((mut map, entities)) = tilemap_query.single_mut() else {
            return;
        };

        let Ok((mut hp, pos, mut kind, mut sprite)) = query.get_mut(event.entity) else {
            continue;
        };

        if hp.is_zero() {
            continue;
        }

        hp.sub(event.damage);

        let crystal = matches!(
            *kind,
            TileKind::CrystalHidden
                | TileKind::Crystal
                | TileKind::CrystalHurt
                | TileKind::CrystalDying
        );
        let metal = matches!(
            *kind,
            TileKind::MetalHidden | TileKind::Metal | TileKind::MetalHurt | TileKind::MetalDying
        );

        // TODO sound
        let amt = if hp.is_zero() {
            particle_settings.kill_amt() / 2
        } else {
            particle_settings.hit_amt() / 2
        };
        for _ in 0..amt {
            commands.spawn((
                ParticleKind::Stone,
                Transform::from_translation(map.pos_to_world(*pos).extend(0.)),
            ));
        }
        for _ in 0..amt {
            commands.spawn((
                if crystal {
                    ParticleKind::Crystal
                } else if metal {
                    ParticleKind::Metal
                } else {
                    ParticleKind::Stone
                },
                Transform::from_translation(map.pos_to_world(*pos).extend(0.)),
            ));
        }

        let health = StoneHealth::from(&*hp);

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

        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = kind.atlas_index();
        }
        map.0[(pos.y, pos.x)] = *kind;

        if hp.is_zero() {
            stats.mined += 1;

            if crystal {
                currency.crystal += 1;
            } else if metal {
                currency.metal += 1;
            } else {
                currency.stone += 1;
            }

            if spawning_paused.0 && (currency.metal > 0 || currency.stone > 4) {
                spawning_paused.0 = false;
            }

            if let Some(designation) = designations.0.remove(pos) {
                commands.entity(designation.indicator).despawn();
            }

            for offset in &crate::pathfinding::NEIGHBORS {
                let neighbor = *offset + *pos;

                let Some(kind) = map.0.get(neighbor.1, neighbor.0) else {
                    continue;
                };

                let Some(Some(entity)) = entities.0.get(neighbor.1, neighbor.0) else {
                    continue;
                };

                if matches!(kind, TileKind::CrystalHidden | TileKind::MetalHidden) {
                    reveal_writer.write(RevealStoneMessage(*entity));
                }
            }
        }
    }
}

fn reveal_events(
    mut messages: MessageReader<RevealStoneMessage>,
    mut query: Query<(&mut TileKind, &mut Sprite)>,
) {
    for message in messages.read() {
        let Ok((mut kind, mut sprite)) = query.get_mut(message.0) else {
            continue;
        };

        *kind = match *kind {
            TileKind::CrystalHidden => TileKind::Crystal,
            TileKind::MetalHidden => TileKind::Metal,
            _ => continue,
        };

        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = kind.atlas_index();
        }
    }
}
