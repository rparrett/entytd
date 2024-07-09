use crate::tilemap::Map;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    utils::BoxedFuture,
};
use image::{GenericImageView, ImageError, Pixel};
use thiserror::Error;

pub struct MapFileLoaderPlugin;
impl Plugin for MapFileLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<MapFileLoader>();
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
    type Asset = Map;
    type Settings = ();
    type Error = MapFileLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut reader = image::io::Reader::new(std::io::Cursor::new(bytes));
        reader.set_format(image::ImageFormat::Png);
        reader.no_limits();
        let dyn_img = reader.decode()?;

        let mut map = Map::new(dyn_img.height() as usize, dyn_img.width() as usize);

        for (x, y, rgba) in dyn_img.pixels() {
            let Ok(kind) = rgba.to_rgb().0.try_into() else {
                continue;
            };

            let inv_y = dyn_img.height() - y - 1;

            map.0[(inv_y as usize, x as usize)] = kind;
        }

        Ok(map)
    }

    fn extensions(&self) -> &[&str] {
        &["map.png"]
    }
}
