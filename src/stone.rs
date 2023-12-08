use bevy::prelude::*;

use crate::{
    currency::Currency,
    designate_tool::Designations,
    hit_points::HitPoints,
    particle::{ParticleBundle, ParticleKind, ParticleSettings},
    tilemap::{TileEntities, TileKind, TilePos, Tilemap},
    GameState,
};

pub struct StonePlugin;
impl Plugin for StonePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HitStoneEvent>()
            .add_event::<RevealStoneEvent>()
            .add_systems(
                Update,
                (hit_events, reveal_events).run_if(in_state(GameState::Playing)),
            );
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
        let fraction = value.fraction();
        if fraction <= 0.25 {
            return Self::Dying;
        } else if fraction < 1.0 {
            return Self::Hurt;
        }

        Self::Full
    }
}

#[derive(Event)]
pub struct HitStoneEvent {
    pub entity: Entity,
    pub damage: u32,
}

#[derive(Event)]
pub struct RevealStoneEvent(Entity);

fn hit_events(
    mut commands: Commands,
    mut reader: EventReader<HitStoneEvent>,
    mut writer: EventWriter<RevealStoneEvent>,
    mut query: Query<(
        &mut HitPoints,
        &TilePos,
        &mut TileKind,
        &mut TextureAtlasSprite,
    )>,
    mut designations: ResMut<Designations>,
    mut tilemap_query: Query<(&mut Tilemap, &TileEntities)>,
    mut currency: ResMut<Currency>,
    particle_settings: Res<ParticleSettings>,
) {
    for event in reader.read() {
        let Ok((mut map, entities)) = tilemap_query.get_single_mut() else {
            return;
        };

        let Ok((mut hp, pos, mut kind, mut sprite)) = query.get_mut(event.entity) else {
            continue;
        };

        if hp.is_zero() {
            continue;
        }

        hp.sub(event.damage);

        // TODO sound
        // TODO obey particle settings
        let amt = if hp.is_zero() {
            particle_settings.kill_amt()
        } else {
            particle_settings.hit_amt()
        };
        for _ in 0..amt {
            commands.spawn(ParticleBundle::new(
                ParticleKind::Stone,
                map.pos_to_world(*pos),
            ));
        }

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
            TileKind::MetalHidden | TileKind::Metal | TileKind::MetalHurt | TileKind::MetalDying
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
            if crystal {
                currency.crystal += 1;
            } else if metal {
                currency.metal += 1;
            } else {
                currency.stone += 1;
            }

            if let Some(designation) = designations.0.remove(pos) {
                commands.entity(designation.indicator).despawn();
            }

            for n in &crate::pathfinding::NEIGHBORS {
                let x = pos.x as isize + n.0;
                let Ok(x) = usize::try_from(x) else {
                    continue;
                };
                let y = pos.y as isize + n.1;
                let Ok(y) = usize::try_from(y) else {
                    continue;
                };
                if x > map.width - 1 || y > map.height - 1 {
                    continue;
                }

                let Some(entity) = entities.entities[x][y] else {
                    continue;
                };

                let kind = map.tiles[x][y];

                if matches!(kind, TileKind::CrystalHidden | TileKind::MetalHidden) {
                    writer.send(RevealStoneEvent(entity));
                }
            }
        }
    }
}

fn reveal_events(
    mut reader: EventReader<RevealStoneEvent>,
    mut query: Query<(&mut TileKind, &mut TextureAtlasSprite)>,
) {
    for event in reader.read() {
        let Ok((mut kind, mut sprite)) = query.get_mut(event.0) else {
            continue;
        };

        *kind = match *kind {
            TileKind::CrystalHidden => TileKind::Crystal,
            TileKind::MetalHidden => TileKind::Metal,
            _ => continue,
        };

        sprite.index = kind.atlas_index();
    }
}
