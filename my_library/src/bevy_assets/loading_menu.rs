use crate::bevy_assets::{setup_asset_store, AssetManager, AssetStore, LoadedAssets};
use crate::bevy_framework::MenuResource;
use bevy::asset::LoadedUntypedAsset;
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
use bevy_egui::egui::Window;
use bevy_egui::EguiContexts;

#[derive(Resource)]
pub(crate) struct AssetsToLoad(Vec<Handle<LoadedUntypedAsset>>);

pub(crate) fn setup_loading_menu(
    assets: Option<Res<AssetStore>>,
    asset_manager: Option<Res<AssetManager>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let assets = match assets {
        Some(assets) => assets.into_inner(),
        None => &setup_asset_store(
            asset_manager.as_ref().unwrap(),
            &mut commands,
            &asset_server,
        ),
    };
    let assets_to_load: Vec<Handle<LoadedUntypedAsset>> =
        assets.asset_index.values().cloned().collect();
    commands.insert_resource(AssetsToLoad(assets_to_load));
}

pub(crate) fn run_loading_menu<T>(
    asset_server: Res<AssetServer>,
    mut to_load: ResMut<AssetsToLoad>,
    mut state: ResMut<NextState<T>>,
    // mut egui_context: EguiContexts,
    menu_info: Res<MenuResource<T>>,
    mut store: ResMut<AssetStore>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_assets: Res<LoadedAssets>,
) where
    T: States + FromWorld + FreelyMutableState,
{
    to_load
        .0
        .retain(|handle| match asset_server.get_load_state(handle.id()) {
            Some(bevy::asset::LoadState::Loaded) => false,
            _ => true,
        });
    if to_load.0.is_empty() {
        load_atlases(&mut store, &mut texture_atlases, &loaded_assets);
        println!("Atlases loaded!");
        state.set(menu_info.menu_state.clone());
    }
    println!("Loading: {} assets remaining", to_load.0.len());
    // if let Ok(ctx) = egui_context.ctx_mut() {
    //     Window::new("Loading, Please Wait").show(ctx, |ui| {
    //         ui.label(format!("Loading: {} assets remaining", to_load.0.len()));
    //     });
    // }
}

pub(crate) fn exit_menu(mut commands: Commands) {
    commands.remove_resource::<AssetsToLoad>();
}

pub(crate) fn load_atlases(
    asset_store: &mut AssetStore,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_assets: &LoadedAssets,
) {
    for new_atlas in asset_store.atlases_to_build.iter() {
        let atlas = TextureAtlasLayout::from_grid(
            new_atlas.tile_size.as_uvec2(),
            new_atlas.sprites_x as u32,
            new_atlas.sprites_y as u32,
            None, None
        );
        let atlas_handle = texture_atlases.add(atlas);
        let img = asset_store.get_handle(&new_atlas.texture_tag, loaded_assets).expect("Could not load texture from teh atlas");
        asset_store.atlases.insert(new_atlas.tag.clone(), (img, atlas_handle));
    }
}
