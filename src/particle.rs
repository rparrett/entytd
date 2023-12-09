use bevy::prelude::*;
use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::{
    layer,
    settings::ParticlesSetting,
    tilemap::{AtlasHandle, SCALE},
    GameState,
};

pub struct ParticlePlugin;
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleRng>()
            .init_resource::<ParticlesSetting>()
            .add_systems(
                Update,
                update_particles.run_if(not(in_state(GameState::Loading))),
            );
    }
}

#[derive(Resource)]
pub struct ParticleRng(SmallRng);
impl Default for ParticleRng {
    fn default() -> Self {
        Self(SmallRng::from_entropy())
    }
}

#[derive(Component, Default)]
pub enum ParticleKind {
    #[default]
    Stone,
    Wood,
    Home,
    Bone,
}
impl ParticleKind {
    fn color(&self) -> Color {
        match self {
            Self::Stone => Color::rgb(0.5, 0.5, 0.5),
            Self::Bone => Color::rgb(0.87, 0.82, 0.76),
            Self::Home => Color::rgb(0.66, 0.38, 0.12),
            Self::Wood => Color::rgb(0.46, 0.25, 0.03),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ParticleBundle {
    sprite: SpriteSheetBundle,
    kind: ParticleKind,
    velocity: Velocity,
    life: Life,
}
impl ParticleBundle {
    pub fn new(kind: ParticleKind, pos: Vec2) -> Self {
        let sprite = SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index: 103 * 49 + 53,
                color: kind.color(),
                ..default()
            },
            transform: Transform::from_translation(pos.extend(layer::PARTICLE))
                .with_scale(SCALE.extend(1.)),
            visibility: Visibility::Hidden,
            ..default()
        };

        ParticleBundle {
            sprite,
            kind,
            ..default()
        }
    }
}

#[derive(Component)]
struct Velocity(Vec2);
impl Default for Velocity {
    fn default() -> Self {
        Self(Vec2::Y * 50.)
    }
}

#[derive(Component)]
struct Life(f32);
impl Default for Life {
    fn default() -> Self {
        Self(1.)
    }
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Life,
        &mut Transform,
        &mut Velocity,
        &mut Handle<TextureAtlas>,
        &mut Visibility,
        Ref<ParticleKind>,
    )>,
    mut rng: ResMut<ParticleRng>,
    atlas_handle: Res<AtlasHandle>,
    time: Res<Time>,
) {
    for (entity, mut life, mut transform, mut velocity, mut handle, mut visibility, kind) in
        &mut query
    {
        if kind.is_added() {
            velocity.0 =
                Vec2::new(rng.0.gen::<f32>() - 0.5, 1.0).normalize() * (rng.0.gen::<f32>() + 1.0);
            *handle = atlas_handle.0.clone();
            *visibility = Visibility::Visible;
            continue;
        }

        let dt = time.delta_seconds();

        life.0 -= dt;
        if life.0 <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        velocity.0 -= Vec2::Y * 10. * dt;

        transform.translation.x += velocity.0.x;
        transform.translation.y += velocity.0.y;
    }
}
