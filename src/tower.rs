use bevy::prelude::*;

use crate::{
    designate_tool::Designations,
    enemy::{Enemy, EnemyKind},
    hit_points::HitPoints,
    layer,
    movement::Speed,
    particle::{ParticleBundle, ParticleKind},
    settings::ParticlesSetting,
    tilemap::{AtlasHandle, Map, TileEntities, TileKind, TilePos, SCALE, TILE_SIZE},
    util::cleanup,
    GameState,
};

pub struct TowerPlugin;
impl Plugin for TowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BuildTowerEvent>()
            .add_systems(
                Update,
                (build_tower, attack, bullet_movement).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                (cleanup::<Tower>, cleanup::<Bullet>),
            );
    }
}

#[derive(Component)]
pub struct Tower;

#[derive(Component, Default)]
struct Upgrades(u32);

#[derive(Component)]
struct CooldownTimer(Timer);

#[derive(Component)]
struct Range(f32);

#[derive(Component)]
struct Bullet {
    damage: u32,
    target: Entity,
}

#[derive(Bundle)]

pub struct TowerBundle {
    sprite: SpriteSheetBundle,
    tower: Tower,
    kind: TileKind,
    cooldown: CooldownTimer,
    upgrades: Upgrades,
    pos: TilePos,
    range: Range,
}
impl Default for TowerBundle {
    fn default() -> Self {
        Self {
            sprite: SpriteSheetBundle::default(),
            tower: Tower,
            kind: TileKind::Tower,
            cooldown: CooldownTimer(Timer::from_seconds(1.0, TimerMode::Once)),
            upgrades: Upgrades::default(),
            pos: TilePos::default(),
            range: Range(TILE_SIZE.x * SCALE.x * 2.),
        }
    }
}

#[derive(Bundle)]
pub struct BulletBundle {
    sprite: SpriteSheetBundle,
    bullet: Bullet,
    speed: Speed,
}

#[derive(Event, Debug)]
pub struct BuildTowerEvent(pub TilePos);

fn attack(
    mut commands: Commands,
    mut query: Query<(&Transform, &Range, &Upgrades, &mut CooldownTimer), With<Tower>>,
    time: Res<Time>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    atlas_handle: Res<AtlasHandle>,
) {
    for (transform, range, upgrades, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if !timer.0.finished() {
            continue;
        }

        let range_sq = range.0 * range.0;
        let pos = transform.translation.truncate();

        for (entity, enemy_transform) in &enemies {
            let enemy_pos = enemy_transform.translation.truncate();
            if pos.distance_squared(enemy_pos) > range_sq {
                continue;
            }

            commands.spawn(BulletBundle {
                sprite: SpriteSheetBundle {
                    atlas: TextureAtlas {
                        layout: atlas_handle.layout.clone(),
                        index: 103 * 49 + 52,
                    },
                    texture: atlas_handle.image.clone(),
                    transform: Transform {
                        scale: SCALE.extend(1.),
                        translation: pos.extend(layer::BULLET),
                        ..default()
                    },
                    ..default()
                },
                bullet: Bullet {
                    damage: 1 + upgrades.0,
                    target: entity,
                },
                speed: Speed(4.),
            });

            timer.0.reset();

            break;
        }
    }
}

fn build_tower(
    mut commands: Commands,
    mut events: EventReader<BuildTowerEvent>,
    mut designations: ResMut<Designations>,
    mut tilemap_query: Query<(&mut Map, &mut TileEntities)>,
    atlas_handle: Res<AtlasHandle>,
) {
    for event in events.read() {
        let Ok((mut tilemap, mut tile_entities)) = tilemap_query.get_single_mut() else {
            continue;
        };

        let world = tilemap.pos_to_world(event.0).extend(layer::BACKGROUND);

        let Some(tile_kind) = tilemap.0.get_mut(event.0.y, event.0.x) else {
            continue;
        };

        if !tile_kind.buildable() {
            continue;
        }

        let Some(maybe_tile_entity) = tile_entities.0.get_mut(event.0.y, event.0.x) else {
            continue;
        };

        // TODO we could probably just insert the bundle onto the existing tile
        // entity.
        if let Some(entity) = maybe_tile_entity.take() {
            commands.entity(entity).despawn();
        }

        let id = commands
            .spawn(TowerBundle {
                sprite: SpriteSheetBundle {
                    atlas: TextureAtlas {
                        layout: atlas_handle.layout.clone(),
                        index: TileKind::Tower.atlas_index(),
                    },
                    texture: atlas_handle.image.clone(),
                    transform: Transform {
                        scale: SCALE.extend(1.),
                        translation: world,
                        ..default()
                    },
                    ..default()
                },
                pos: event.0,
                ..default()
            })
            .id();

        *maybe_tile_entity = Some(id);
        *tile_kind = TileKind::Tower;

        if let Some(designation) = designations.0.remove(&event.0) {
            commands.entity(designation.indicator).despawn();
        }
    }
}

fn bullet_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &Bullet, &Speed, &mut Transform)>,
    mut enemy_query: Query<
        (&mut HitPoints, &Transform, &EnemyKind),
        (With<Enemy>, Without<Bullet>),
    >,
    time: Res<Time>,
    particle_settings: Res<ParticlesSetting>,
) {
    for (bullet_entity, bullet, speed, mut transform) in query.iter_mut() {
        let Ok((mut hp, enemy, enemy_kind)) = enemy_query.get_mut(bullet.target) else {
            commands.entity(bullet_entity).despawn();
            continue;
        };

        let diff = enemy.translation.truncate() - transform.translation.truncate();
        let dist = diff.length();
        let dir = diff.normalize();
        let step = time.delta_seconds() * speed.0 * TILE_SIZE.x * SCALE.x;

        if dist > step {
            transform.translation.x += step * dir.x;
            transform.translation.y += step * dir.y;
        } else {
            hp.sub(bullet.damage);

            // TODO sound
            let amt = if hp.is_zero() {
                particle_settings.kill_amt() / 2
            } else {
                particle_settings.hit_amt() / 2
            };
            for _ in 0..amt {
                commands.spawn(ParticleBundle::new(
                    match enemy_kind {
                        EnemyKind::Ent | EnemyKind::EntTwo | EnemyKind::EntThree => {
                            ParticleKind::Wood
                        }
                        EnemyKind::EntFour => ParticleKind::Purple,
                        EnemyKind::Skeleton
                        | EnemyKind::SkeletonTwo
                        | EnemyKind::SkeletonThree
                        | EnemyKind::SkeletonFour => ParticleKind::Bone,
                    },
                    enemy.translation.truncate(),
                ));
                commands.spawn(ParticleBundle::new(
                    ParticleKind::Stone,
                    enemy.translation.truncate(),
                ));
            }

            commands.entity(bullet_entity).despawn();
        }
    }
}
