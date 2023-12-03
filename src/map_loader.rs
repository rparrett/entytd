use crate::tilemap::{TileKind, Tilemap};
use bevy::utils::thiserror;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::BoxedFuture,
};
use image::{GenericImageView, ImageError, Pixel};
use thiserror::Error;

pub struct MapFileLoaderPlugin;
impl Plugin for MapFileLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Tilemap>()
            .init_asset_loader::<MapFileLoader>();
    }
}

#[derive(Default)]
pub struct MapFileLoader;

/// Possible errors that can be produced by [`MapFileLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum MapFileLoaderError {
    #[error("Could not load texture: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load image file: {0}")]
    Image(#[from] ImageError),
}

impl AssetLoader for MapFileLoader {
    type Asset = Tilemap;
    type Settings = ();
    type Error = MapFileLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let mut reader = image::io::Reader::new(std::io::Cursor::new(bytes));
            reader.set_format(image::ImageFormat::Png);
            reader.no_limits();
            let dyn_img = reader.decode()?;

            let mut map = Tilemap::new(dyn_img.width() as usize, dyn_img.height() as usize);

            for (x, y, rgba) in dyn_img.pixels() {
                info!("{} {} {:?}", x, y, rgba);
                let Ok(kind) = rgba.to_rgb().0.try_into() else {
                    continue;
                };

                map.tiles[x as usize][dyn_img.height() as usize - y as usize - 1].kind = kind;
            }

            Ok(map)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map.png"]
    }
}
