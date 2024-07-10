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
    Crystal,
    Metal,
    Purple,
}
impl ParticleKind {
    fn color(&self) -> Color {
        match self {
            Self::Stone => Color::srgb(0.5, 0.5, 0.5),
            Self::Bone => Color::srgb(0.87, 0.82, 0.76),
            Self::Home => Color::srgb(0.66, 0.38, 0.12),
            Self::Wood => Color::srgb(0.46, 0.25, 0.03),
            Self::Crystal => Color::srgb(0.37, 0.69, 0.84),
            Self::Metal => Color::srgb(0.84, 0.73, 0.37),
            Self::Purple => Color::srgb(0.74, 0., 0.71),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ParticleBundle {
    sprite: SpriteBundle,
    texture: TextureAtlas,
    kind: ParticleKind,
    velocity: Velocity,
    life: Life,
}
impl ParticleBundle {
    pub fn new(kind: ParticleKind, pos: Vec2) -> Self {
        ParticleBundle {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: kind.color(),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(layer::PARTICLE))
                    .with_scale(SCALE.extend(1.)),
                visibility: Visibility::Hidden,
                ..default()
            },
            texture: TextureAtlas {
                index: 103 * 49 + 53,
                ..default()
            },
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
        &mut TextureAtlas,
        &mut Handle<Image>,
        &mut Visibility,
        Ref<ParticleKind>,
    )>,
    mut rng: ResMut<ParticleRng>,
    atlas_handle: Res<AtlasHandle>,
    time: Res<Time>,
) {
    for (
        entity,
        mut life,
        mut transform,
        mut velocity,
        mut atlas,
        mut image_handle,
        mut visibility,
        kind,
    ) in &mut query
    {
        if kind.is_added() {
            velocity.0 =
                Vec2::new(rng.0.gen::<f32>() - 0.5, 1.0).normalize() * (rng.0.gen::<f32>() + 1.0);
            atlas.layout = atlas_handle.layout.clone();
            *image_handle = atlas_handle.image.clone();
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
