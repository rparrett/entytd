use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

pub struct TilemapPlugin;
impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileAtlas>();
    }
}

#[derive(Resource)]
pub struct TileAtlas(pub Handle<TextureAtlas>);

impl FromWorld for TileAtlas {
    fn from_world(world: &mut World) -> Self {
        let server = world.resource::<AssetServer>();

        let texture_handle = server.load("urizen_onebit_tileset__v1d0.png");
        let atlas_handle = TextureAtlas::from_grid(
            texture_handle,
            Vec2::new(12.0, 12.0),
            103,
            50,
            Some(Vec2::splat(1.)),
            Some(Vec2::splat(1.)),
        );

        let mut atlases = world.resource_mut::<Assets<TextureAtlas>>();

        Self(atlases.add(atlas_handle))
    }
}

#[derive(Clone)]
pub enum TileKind {
    Empty,
    Stone,
    StoneTunnel,
    HurtStone,
    DyingStone,
    Wall,
}

impl Distribution<TileKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TileKind {
        match rng.gen_range(0..=4) {
            0 => TileKind::Empty,
            1 => TileKind::Stone,
            2 => TileKind::HurtStone,
            3 => TileKind::DyingStone,
            _ => TileKind::Wall,
        }
    }
}
impl TileKind {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Empty => 16,
            Self::StoneTunnel => 210,
            Self::Stone => 211,
            Self::HurtStone => 212,
            Self::DyingStone => 213,
            Self::Wall => 309,
        }
    }
}

#[derive(Clone)]
pub struct Tile {
    kind: TileKind,
    sprite: Option<Entity>,
}

pub struct Tilemap {
    tiles: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl Tilemap {
    pub fn new_random(width: usize, height: usize) -> Self {
        let mut rng = thread_rng();

        let mut map = Self {
            tiles: vec![
                vec![
                    Tile {
                        kind: TileKind::Empty,
                        sprite: None
                    };
                    height
                ];
                width
            ],
            width,
            height,
        };

        for x in 0..width {
            map.tiles.push(Vec::with_capacity(height));

            for y in 0..height {
                let kind: TileKind = rng.gen();

                map.tiles[x][y] = Tile { kind, sprite: None };
            }
        }

        map
    }

    pub fn spawn(&mut self, commands: &mut Commands, atlas_handle: Handle<TextureAtlas>) {
        let scale = 2.;
        let tile_size = 12.;

        for x in 0..self.width {
            for y in 0..self.height {
                let tile = &mut self.tiles[x][y];

                let entity = commands
                    .spawn(SpriteSheetBundle {
                        texture_atlas: atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(tile.kind.atlas_index()),
                        transform: Transform::from_scale(Vec3::splat(scale)).with_translation(
                            Vec3::new(
                                scale * tile_size * (-(self.width as f32) / 2. + x as f32)
                                    + tile_size / 2. * scale,
                                scale * tile_size * (-(self.height as f32) / 2. + y as f32)
                                    + tile_size / 2. * scale,
                                0.,
                            ),
                        ),
                        ..default()
                    })
                    .id();

                tile.sprite = Some(entity);
            }
        }
    }
}
