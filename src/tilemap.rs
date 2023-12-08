use bevy::prelude::*;

use crate::{
    hit_points::HitPoints,
    home::Home,
    layer,
    level::{LevelConfig, LevelHandle},
    loading::LoadingAssets,
    spawner::{Spawner, SpawnerIndex},
    GameState,
};

pub struct TilemapPlugin;
impl Plugin for TilemapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TileKind>()
            .register_type::<TilePos>()
            .add_systems(Update, queue_load.run_if(in_state(GameState::Loading)))
            .add_systems(Update, process_loaded_maps)
            .add_systems(OnEnter(GameState::Playing), spawn);
    }
}

pub const SCALE: Vec2 = Vec2::splat(2.);
pub const TILE_SIZE: Vec2 = Vec2::splat(12.);

#[derive(Reflect, Debug, Component, Clone, Copy)]
pub enum TileKind {
    Empty,
    Stone,
    StoneTunnel,
    StoneHurt,
    StoneDying,
    CrystalHidden,
    CrystalHurt,
    CrystalDying,
    Crystal,
    MetalHidden,
    MetalHurt,
    MetalDying,
    Metal,
    Wall,
    Mountain,
    Peak,
    Volcano,
    River,
    Forest,
    GrassA,
    GrassB,
    Home,
    HomeTwo,
    Dirt,
    Bridge,
    Road,
    Spawn,
    Tower,
    TowerBlueprint,
    White,
    WhitePickaxe,
    WhiteCircleNo,
    DirtPathNSA,
    DirtPathNSB,
    DirtPathEWA,
    DirtPathEWB,
    DirtPathSW,
    DirtPathNW,
    DirtPathSE,
    DirtPathNE,
    DirtPathNSW,
    DirtPathSEW,
    DirtPathNSE,
    DirtPathNEW,
    DirtPathNSEW,
    Stump,
    HomeDead,
    BonesA,
    BonesB,
    BonesC,
    BonesD,
}

pub struct TileNotMappedError;

impl TryFrom<[u8; 3]> for TileKind {
    type Error = TileNotMappedError;

    fn try_from(value: [u8; 3]) -> Result<Self, Self::Error> {
        match value {
            [0, 0, 0] => Ok(TileKind::Empty),
            [10, 10, 10] => Ok(Self::Dirt),
            [255, 255, 255] => Ok(TileKind::Stone),
            [235, 235, 235] => Ok(TileKind::CrystalHidden),
            [225, 225, 225] => Ok(TileKind::MetalHidden),
            [200, 0, 0] => Ok(Self::Mountain),
            [255, 0, 0] => Ok(Self::Peak),
            [150, 0, 0] => Ok(Self::Volcano),
            [0, 255, 0] => Ok(Self::Forest),
            [0, 200, 0] => Ok(Self::GrassA),
            [0, 180, 0] => Ok(Self::GrassB),
            [0, 0, 255] => Ok(Self::River),
            [255, 0, 255] => Ok(Self::StoneTunnel),
            [255, 155, 0] => Ok(Self::Home),
            [255, 156, 0] => Ok(Self::HomeTwo),
            [255, 255, 0] => Ok(Self::Bridge),
            [255, 200, 0] => Ok(Self::Road),
            [0, 255, 255] => Ok(Self::Spawn),
            [255, 180, 0] => Ok(Self::DirtPathNSA),
            [255, 180, 10] => Ok(Self::DirtPathNSB),
            [255, 180, 20] => Ok(Self::DirtPathEWA),
            [255, 180, 30] => Ok(Self::DirtPathEWB),
            [255, 180, 40] => Ok(Self::DirtPathSW),
            [255, 180, 50] => Ok(Self::DirtPathNW),
            [255, 180, 60] => Ok(Self::DirtPathSE),
            [255, 180, 70] => Ok(Self::DirtPathNE),
            [255, 180, 80] => Ok(Self::DirtPathNSW),
            [255, 180, 90] => Ok(Self::DirtPathSEW),
            [255, 180, 100] => Ok(Self::DirtPathNSE),
            [255, 180, 110] => Ok(Self::DirtPathNEW),
            [255, 180, 120] => Ok(Self::DirtPathNSEW),
            [100, 100, 50] => Ok(Self::Stump),
            [255, 100, 0] => Ok(Self::BonesA),
            [255, 110, 0] => Ok(Self::BonesB),
            [255, 120, 0] => Ok(Self::BonesC),
            [255, 130, 0] => Ok(Self::BonesD),
            _ => Err(TileNotMappedError),
        }
    }
}

impl TileKind {
    pub fn atlas_index(&self) -> usize {
        match self {
            Self::Empty => 16,
            Self::StoneTunnel => 210,
            Self::Stone | Self::CrystalHidden | Self::MetalHidden => 211,
            Self::StoneHurt => 212,
            Self::StoneDying => 213,
            Self::Crystal => 214,
            Self::CrystalHurt => 215,
            Self::CrystalDying => 216,
            Self::Metal => 217,
            Self::MetalHurt => 218,
            Self::MetalDying => 219,
            Self::Wall => 309,
            Self::Mountain => 103 * 34 + 15,
            Self::Peak => 103 * 34 + 17,
            Self::Volcano => 103 * 34 + 16,
            Self::River => 103 * 34 + 9,
            Self::Forest => 103 * 34 + 2,
            Self::Home => 103 * 33 + 24,
            Self::HomeTwo => 103 * 33 + 23,
            Self::HomeDead => 103 * 33 + 22,
            Self::Dirt => 103 * 5 + 1,
            Self::Bridge => 103 * 4,
            Self::Road => 103 + 13,
            Self::Spawn => 103 * 17 + 80,
            Self::Tower => 103 * 22 + 42,
            Self::TowerBlueprint => 103 * 22 + 43,
            Self::White => 103 * 48 + 52,
            Self::WhitePickaxe => 103 * 48 + 53,
            Self::WhiteCircleNo => 103 * 48 + 54,
            Self::DirtPathNSA => 103 * 11,
            Self::DirtPathNSB => 103 * 11 + 1,
            Self::DirtPathEWA => 103 * 11 + 2,
            Self::DirtPathEWB => 103 * 11 + 3,
            Self::DirtPathSW => 103 * 11 + 4,
            Self::DirtPathNW => 103 * 11 + 5,
            Self::DirtPathSE => 103 * 11 + 6,
            Self::DirtPathNE => 103 * 11 + 7,
            Self::DirtPathNSW => 103 * 11 + 8,
            Self::DirtPathSEW => 103 * 11 + 9,
            Self::DirtPathNSE => 103 * 11 + 10,
            Self::DirtPathNEW => 103 * 11 + 11,
            Self::DirtPathNSEW => 103 * 11 + 12,
            Self::GrassA => 103 * 9 + 3,
            Self::GrassB => 103 * 9 + 4,
            Self::Stump => 103 * 10 + 4,
            Self::BonesA => 103 * 38,
            Self::BonesB => 103 * 38 + 1,
            Self::BonesC => 103 * 38 + 2,
            Self::BonesD => 103 * 38 + 3,
        }
    }
}

impl TileKind {
    pub fn diggable(&self) -> bool {
        matches!(
            self,
            TileKind::Stone
                | TileKind::StoneDying
                | TileKind::StoneHurt
                | TileKind::Crystal
                | TileKind::CrystalHidden
                | TileKind::CrystalHurt
                | TileKind::CrystalDying
                | TileKind::Metal
                | TileKind::MetalHidden
                | TileKind::MetalHurt
                | TileKind::MetalDying
        )
    }
    pub fn buildable(&self) -> bool {
        matches!(self, TileKind::Dirt)
    }
}

#[derive(Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub sprite: Option<Entity>,
}

#[derive(Component, Asset, TypePath, Default, Clone)]
pub struct Tilemap {
    pub tiles: Vec<Vec<TileKind>>,
    pub width: usize,
    pub height: usize,
}
impl Tilemap {
    pub fn size_vec2(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    pub fn pos_to_world(&self, pos: TilePos) -> Vec2 {
        let size = self.size_vec2();
        let pos: Vec2 = pos.into();
        let pos = pos - size / 2.;

        SCALE * TILE_SIZE * pos + TILE_SIZE / 2. * SCALE
    }

    pub fn world_to_pos(&self, world: Vec2) -> TilePos {
        let size = self.size_vec2() * SCALE * TILE_SIZE;

        let pos = (world + size / 2. - TILE_SIZE / 2. * SCALE) / TILE_SIZE / SCALE;

        TilePos {
            x: pos.x.round() as usize,
            y: pos.y.round() as usize,
        }
    }

    pub fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![vec![TileKind::Empty; height]; width],
            width,
            height,
        }
    }

    pub fn get(&self, pos: TilePos) -> Option<&TileKind> {
        let col = self.tiles.get(pos.x)?;
        col.get(pos.y)
    }

    pub fn get_mut(&mut self, pos: TilePos) -> Option<&mut TileKind> {
        let col = self.tiles.get_mut(pos.x)?;
        col.get_mut(pos.y)
    }
}

// TODO argh why is this not just a hashmap
#[derive(Component, Default)]
pub struct TileEntities {
    pub entities: Vec<Vec<Option<Entity>>>,
}
impl TileEntities {
    #[allow(unused)]
    pub fn get(&self, pos: TilePos) -> Option<&Option<Entity>> {
        let col = self.entities.get(pos.x)?;
        col.get(pos.y)
    }

    pub fn get_mut(&mut self, pos: TilePos) -> Option<&mut Option<Entity>> {
        let col = self.entities.get_mut(pos.x)?;
        col.get_mut(pos.y)
    }
}

#[derive(Reflect, Component, Debug, Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct TilePos {
    pub x: usize,
    pub y: usize,
}
impl Into<Vec2> for TilePos {
    fn into(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}
impl Into<IVec2> for TilePos {
    fn into(self) -> IVec2 {
        IVec2::new(self.x as i32, self.y as i32)
    }
}
impl From<(isize, isize)> for TilePos {
    fn from(value: (isize, isize)) -> Self {
        TilePos {
            x: value.0 as usize,
            y: value.1 as usize,
        }
    }
}
impl From<(usize, usize)> for TilePos {
    fn from(value: (usize, usize)) -> Self {
        TilePos {
            x: value.0,
            y: value.1,
        }
    }
}
impl Into<(isize, isize)> for TilePos {
    fn into(self) -> (isize, isize) {
        (self.x as isize, self.y as isize)
    }
}

#[derive(Resource)]
pub struct TilemapHandle(pub Handle<Tilemap>);

#[derive(Resource)]
pub struct AtlasHandle(pub Handle<TextureAtlas>);

#[derive(Bundle, Default)]
pub struct TilemapBundle {
    tilemap_handle: Handle<Tilemap>,
    atlas_handle: Handle<TextureAtlas>,
    tiles: Tilemap,
    entities: TileEntities,
}

fn queue_load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    level_handle: Option<Res<LevelHandle>>,
    levels: Res<Assets<LevelConfig>>,
    mut queued: Local<bool>,
) {
    if *queued {
        return;
    }

    let Some(level_handle) = level_handle else {
        return;
    };

    let Some(level) = levels.get(&level_handle.0) else {
        return;
    };

    let tilemap_handle = asset_server.load(&level.map);

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

    commands.insert_resource(TilemapHandle(tilemap_handle));
    commands.insert_resource(AtlasHandle(atlas_handle));

    *queued = true;
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

            let mut spawner_index = 0;

            for x in 0..map.width {
                for y in 0..map.height {
                    let tile = &map.tiles[x][y];

                    let mut command = commands.spawn((
                        SpriteSheetBundle {
                            texture_atlas: atlas_handle.clone(),
                            sprite: TextureAtlasSprite::new(tile.atlas_index()),
                            transform: Transform {
                                scale: SCALE.extend(1.),
                                translation: map
                                    .pos_to_world(TilePos { x, y })
                                    .extend(layer::BACKGROUND),
                                ..default()
                            },
                            ..default()
                        },
                        TilePos { x, y },
                        *tile,
                        #[cfg(feature = "inspector")]
                        Name::new("Tile"),
                    ));

                    match tile {
                        TileKind::Spawn => {
                            command.insert((
                                Spawner,
                                SpawnerIndex(spawner_index),
                                #[cfg(feature = "inspector")]
                                Name::new("SpawnerTile"),
                            ));

                            spawner_index += 1;
                        }
                        TileKind::Home | TileKind::HomeTwo => {
                            command.insert((
                                Home,
                                HitPoints::full(30),
                                #[cfg(feature = "inspector")]
                                Name::new("HomeTile"),
                            ));
                        }
                        TileKind::Stone => {
                            command.insert((
                                HitPoints::full(8),
                                #[cfg(feature = "inspector")]
                                Name::new("StoneTile"),
                            ));
                        }
                        TileKind::CrystalHidden | TileKind::MetalHidden => {
                            command.insert((
                                HitPoints::full(24),
                                #[cfg(feature = "inspector")]
                                Name::new("ResourceTile"),
                            ));
                        }
                        _ => {}
                    }

                    let entity = command.id();

                    tile_entities.entities[x][y] = Some(entity);
                }
            }
        }
    }
}

fn spawn(
    mut commands: Commands,
    tilemap_handle: Res<TilemapHandle>,
    atlas_handle: Res<AtlasHandle>,
    tilemaps: Res<Assets<Tilemap>>,
) {
    commands.spawn(TilemapBundle {
        tilemap_handle: tilemap_handle.0.clone(),
        atlas_handle: atlas_handle.0.clone(),
        tiles: tilemaps.get(&tilemap_handle.0).unwrap().clone(),
        ..default()
    });
}
