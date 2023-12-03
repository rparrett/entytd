use bevy::prelude::*;
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

use crate::{loading::LoadingAssets, GameState};

pub struct TilemapPlugin;
impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileAtlas>()
            .add_systems(OnEnter(GameState::Loading), queue_load)
            .add_systems(OnEnter(GameState::Playing), spawn);
    }
}

pub const SCALE: f32 = 2.;
pub const TILE_SIZE: f32 = 12.;

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
    Mountain,
    Peak,
    Volcano,
    River,
    Forest,
    Home,
    HomeTwo,
    Dirt,
    Bridge,
    Road,
}

pub struct TileNotMappedError;

impl TryFrom<[u8; 3]> for TileKind {
    type Error = TileNotMappedError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match value {
            [0, 0, 0] => Ok(TileKind::Empty),
            [10, 10, 10] => Ok(Self::Dirt),
            [255, 255, 255] => Ok(TileKind::Stone),
            [200, 0, 0] => Ok(Self::Mountain),
            [255, 0, 0] => Ok(Self::Peak),
            [150, 0, 0] => Ok(Self::Volcano),
            [0, 255, 0] => Ok(Self::Forest),
            [0, 0, 255] => Ok(Self::River),
            [255, 0, 255] => Ok(Self::StoneTunnel),
            [255, 155, 0] => Ok(Self::Home),
            [255, 156, 0] => Ok(Self::HomeTwo),
            [255, 255, 0] => Ok(Self::Bridge),
            [255, 200, 0] => Ok(Self::Road),
            _ => Err(TileNotMappedError),
        }
    }
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
            Self::Mountain => 103 * 34 + 15,
            Self::Peak => 103 * 34 + 17,
            Self::Volcano => 103 * 34 + 16,
            Self::River => 103 * 34 + 9,
            Self::Forest => 103 * 34 + 2,
            Self::Home => 103 * 33 + 24,
            Self::HomeTwo => 103 * 33 + 23,
            Self::Dirt => 103 * 5 + 1,
            Self::Bridge => 103 * 4 + 0,
            Self::Road => 103 * 1 + 13,
        }
    }
}

#[derive(Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub sprite: Option<Entity>,
}

#[derive(Resource, Asset, TypePath)]
pub struct Tilemap {
    pub tiles: Vec<Vec<Tile>>,
    pub width: usize,
    pub height: usize,
}

impl Tilemap {
    pub fn new(width: usize, height: usize) -> Self {
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

        map
    }

    pub fn new_random(width: usize, height: usize) -> Self {
        let mut rng = thread_rng();

        let mut map = Self::new(width, height);

        for x in 0..width {
            for y in 0..height {
                let kind: TileKind = rng.gen();

                map.tiles[x][y] = Tile { kind, sprite: None };
            }
        }

        map
    }

    pub fn spawn(&mut self, commands: &mut Commands, atlas_handle: Handle<TextureAtlas>) {
        for x in 0..self.width {
            for y in 0..self.height {
                let tile = &mut self.tiles[x][y];

                let entity = commands
                    .spawn(SpriteSheetBundle {
                        texture_atlas: atlas_handle.clone(),
                        sprite: TextureAtlasSprite::new(tile.kind.atlas_index()),
                        transform: Transform::from_scale(Vec3::splat(SCALE)).with_translation(
                            Vec3::new(
                                SCALE * TILE_SIZE * (-(self.width as f32) / 2. + x as f32)
                                    + TILE_SIZE / 2. * SCALE,
                                SCALE * TILE_SIZE * (-(self.height as f32) / 2. + y as f32)
                                    + TILE_SIZE / 2. * SCALE,
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

#[derive(Resource)]
pub struct TilemapHandle(pub Handle<Tilemap>);

fn queue_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let handle = asset_server.load("map.map.png");
    commands.insert_resource(TilemapHandle(handle.clone()));
    loading_assets.0.push(handle.into());
}

fn spawn(
    mut commands: Commands,
    atlas: Res<TileAtlas>,
    mut tilemaps: ResMut<Assets<Tilemap>>,
    tilemap_handle: Res<TilemapHandle>,
) {
    info!("spawning tilemap?");
    let map = tilemaps.get_mut(&tilemap_handle.0).unwrap();
    map.spawn(&mut commands, atlas.0.clone());
}
