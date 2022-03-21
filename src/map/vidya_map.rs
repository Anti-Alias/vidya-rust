use std::path::PathBuf;
use bevy::reflect::TypeUuid;
use bevy::asset::{AssetLoader, AssetServerSettings, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use tiled::{Map, FilesystemResourceCache, Loader};

#[derive(Debug, TypeUuid)]
#[uuid = "24740238-86b8-11ec-a8a3-0242ac120002"]
pub struct VidyaMap {
    pub tiled_map: Map
}

pub struct VidyaMapLoader {
    assets_folder: PathBuf
}

impl FromWorld for VidyaMapLoader {
    fn from_world(world: &mut World) -> Self {
        let asset_folder = &world
            .get_resource::<AssetServerSettings>()
            .unwrap()
            .asset_folder;
        Self {
            assets_folder: PathBuf::from(asset_folder)
        }
    }
}

impl AssetLoader for VidyaMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext
    ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut path = PathBuf::new();
            path.push(&self.assets_folder);
            path.push(load_context.path());
            let mut loader = Loader::with_cache(FilesystemResourceCache::new());
            let tiled_map = loader.load_tmx_map_from(bytes, &path).unwrap();
            load_context.set_default_asset(LoadedAsset::new(VidyaMap { tiled_map }));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}