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
            .add_systems(Update, process_loaded_maps);
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

#[derive(Asset, TypePath)]
pub struct Tilemap {
    pub tiles: Vec<Vec<TileKind>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Component, Default)]
pub struct TileEntities {
    pub entities: Vec<Vec<Option<Entity>>>,
}

impl Tilemap {
    pub fn new(width: usize, height: usize) -> Self {
        let map = Self {
            tiles: vec![vec![TileKind::Empty; height]; width],
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

                map.tiles[x][y] = kind;
            }
        }

        map
    }
}

#[derive(Resource)]
pub struct TilemapHandle(pub Handle<Tilemap>);

#[derive(Bundle, Default)]
pub struct TilemapBundle {
    tilemap_handle: Handle<Tilemap>,
    atlas_handle: Handle<TextureAtlas>,
    tiles: TileEntities,
}

fn queue_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let tilemap_handle = asset_server.load("map.map.png");

    let texture_handle = asset_server.load("urizen_onebit_tileset__v1d0.png");
    loading_assets.0.push(texture_handle.clone().into());

    let atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(12.0, 12.0),
        103,
        50,
        Some(Vec2::splat(1.)),
        Some(Vec2::splat(1.)),
    );

    let atlas_handle = atlases.add(atlas);

    loading_assets.0.push(tilemap_handle.clone().into());

    // TODO save the handles and spawn after load

    commands.spawn(TilemapBundle {
        tilemap_handle,
        atlas_handle,
        ..default()
    });
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<Tilemap>>,
    maps: Res<Assets<Tilemap>>,
    mut map_query: Query<(&Handle<Tilemap>, &Handle<TextureAtlas>, &mut TileEntities)>,
    new_maps: Query<&Handle<Tilemap>, Added<Handle<Tilemap>>>,
) {
    let mut changed_maps = Vec::<AssetId<Tilemap>>::default();
    for event in map_events.read() {
        match event {
            AssetEvent::Added { id } => {
                info!("Map added.");
                changed_maps.push(*id);
            }
            AssetEvent::Modified { id } => {
                info!("Map changed.");
                changed_maps.push(*id);
            }
            AssetEvent::Removed { id } => {
                info!("Map removed.");

                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == id);
            }
            _ => continue,
        }
    }
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.id());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, atlas_handle, mut tile_entities) in map_query.iter_mut() {
            if map_handle.id() != *changed_map {
                continue;
            }

            for entity in tile_entities.entities.iter().flatten().flatten() {
                commands.entity(*entity).despawn_recursive();
            }

            let Some(map) = maps.get(map_handle) else {
                continue;
            };

            tile_entities.entities = vec![vec![None; map.height]; map.width];

            for x in 0..map.width {
                for y in 0..map.height {
                    let tile = &map.tiles[x][y];

                    let entity = commands
                        .spawn(SpriteSheetBundle {
                            texture_atlas: atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(tile.atlas_index()),
                            transform: Transform::from_scale(Vec3::splat(SCALE)).with_translation(
                                Vec3::new(
                                    SCALE * TILE_SIZE * (-(map.width as f32) / 2. + x as f32)
                                        + TILE_SIZE / 2. * SCALE,
                                    SCALE * TILE_SIZE * (-(map.height as f32) / 2. + y as f32)
                                        + TILE_SIZE / 2. * SCALE,
                                    0.,
                                ),
                            ),
                            ..default()
                        })
                        .id();

                    tile_entities.entities[x][y] = Some(entity);
                }
            }
        }
    }
}
