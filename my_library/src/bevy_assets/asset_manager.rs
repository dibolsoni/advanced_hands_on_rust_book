use crate::bevy_assets::asset_store::AssetStore;
use anyhow::anyhow;
use bevy::app::Plugin;
use bevy::asset::AssetServer;
use bevy::platform::collections::HashMap;
use bevy::prelude::{Commands, Res, Resource, Vec2};
use crate::bevy_assets::FutureAtlas;

#[derive(Clone)]
pub enum AssetType {
    Image,
    Sound,
    SpriteSheet {
        tile_size: Vec2,
        sprites_x: usize,
        sprites_y: usize,
    },
}

#[derive(Resource, Clone)]
pub struct AssetManager {
    asset_list: Vec<(String, String, AssetType)>,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            asset_list: vec![
                (
                    "main_menu".to_string(),
                    "main_menu.png".to_string(),
                    AssetType::Image,
                ),
                (
                    "game_over".to_string(),
                    "game_over.png".to_string(),
                    AssetType::Image,
                ),
            ],
        }
    }

    fn asset_exists(filename: &str) -> anyhow::Result<()> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let current_directory = std::env::current_dir()?;
            let assets = current_directory.join("flappy_dragon/assets");
            let new_image = assets.join(filename);
            if !new_image.exists() {
                return Err(anyhow!("Asset {:?} does not exist", &new_image));
            };
            Ok(())
        }
    }

    pub fn add_image<S: ToString>(mut self, tag: S, filename: S) -> anyhow::Result<Self> {
        println!("Loading image {:?}", filename.to_string());
        let filename = filename.to_string();
        AssetManager::asset_exists(&filename)?;
        self.asset_list
            .push((tag.to_string(), filename, AssetType::Image));
        Ok(self)
    }

    pub fn add_sound<S: ToString>(mut self, tag: S, filename: S) -> anyhow::Result<Self> {
        println!("Loading sound {:?}", filename.to_string());
        let filename = filename.to_string();
        AssetManager::asset_exists(&filename)?;
        self.asset_list
            .push((tag.to_string(), filename, AssetType::Sound));
        Ok(self)
    }

    pub fn add_spritesheet<S: ToString>(
        mut self,
        tag: S,
        filename: S,
        sprite_width: f32,
        sprite_height: f32,
        sprites_x: usize,
        sprites_y: usize,
    ) -> anyhow::Result<Self> {
        let filename = filename.to_string();
        AssetManager::asset_exists(&filename)?;
        self.asset_list.push((
            tag.to_string(),
            filename,
            AssetType::SpriteSheet {
                tile_size: Vec2::new(sprite_width, sprite_height),
                sprites_x,
                sprites_y,
            },
        ));
        Ok(self)
    }
}

impl Plugin for AssetManager {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.clone());
        // app.add_systems(Startup, setup_asset_store);
    }
}

pub fn setup_asset_store(
    asset_resource: &Res<AssetManager>,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) -> AssetStore {
    println!("Loading AssetStore...");
    let mut assets = AssetStore {
        asset_index: HashMap::new(),
        atlases_to_build: vec![],
        atlases: HashMap::new(),
    };

    asset_resource
        .asset_list
        .iter()
        .for_each(|(tag, filename, asset_type)| match asset_type {
            AssetType::SpriteSheet {tile_size, sprites_x, sprites_y} => {
                let image_handle = asset_server.load_untyped(filename);
                let base_tag = format!("{}_base", tag);
                assets.asset_index.insert(base_tag.clone(), image_handle);

                assets.atlases_to_build.push(FutureAtlas {
                    tag: tag.clone(),
                    texture_tag: base_tag,
                    tile_size: *tile_size,
                    sprites_x: *sprites_x,
                    sprites_y: *sprites_y,
                })
            }
            _ => {
                assets
                    .asset_index
                    .insert(tag.clone(), asset_server.load_untyped(filename));
            }
        });
    commands.remove_resource::<AssetManager>();
    commands.insert_resource(assets.clone());
    assets
}
