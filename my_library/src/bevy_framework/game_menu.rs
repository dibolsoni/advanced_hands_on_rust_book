use bevy::app::AppExit;
use bevy::asset::AssetServer;
use bevy::input::ButtonInput;
use bevy::log;
use bevy::prelude::{
    default, Camera2d, Commands, Component, FromWorld, Handle, Image, KeyCode, MessageWriter,
    NextState, Res, ResMut, Resource, Sprite, State, States, Transform,
};
use bevy::state::state::FreelyMutableState;
use crate::bevy_assets::{AssetResource, AssetStore};

#[derive(Resource)]
pub(crate) struct MenuResource<T> {
    pub(crate) menu_state: T,
    pub(crate) game_start_state: T,
    pub(crate) game_end_state: T,
}

#[derive(Component)]
pub(crate) struct MenuElement;

pub fn setup_menu<T>(
    state: Res<State<T>>,
    mut commands: Commands,
    menu_resource: Res<MenuResource<T>>,
    loaded_assets: AssetResource,
    assets: Res<AssetStore>
) where
    T: States + FromWorld + FreelyMutableState,
{
    let current_state = state.get();
    let menu_graphic = {
        if menu_resource.menu_state == *current_state {
            assets.get_handle("main_menu", &loaded_assets).unwrap()
        } else if menu_resource.game_end_state == *current_state {
            assets.get_handle("game_over", &loaded_assets).unwrap()
        } else {
            panic!("Invalid state for menu graphic");
        }
    };
    commands.spawn(Camera2d::default()).insert(MenuElement);
    commands
        .spawn((
            Sprite {
                image: menu_graphic,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .insert(MenuElement);
}
pub(crate) fn run<T>(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    current_state: ResMut<State<T>>,
    mut state: ResMut<NextState<T>>,
    menu_state: Res<MenuResource<T>>,
) where
    T: States + FromWorld + FreelyMutableState,
{
    let current_state = current_state.get().clone();
    if current_state == menu_state.menu_state {
        if keyboard.just_pressed(KeyCode::KeyP) {
            state.set(menu_state.game_start_state.clone())
        } else if keyboard.just_pressed(KeyCode::KeyQ) {
            exit.write(AppExit::Success);
        }
    } else if current_state == menu_state.game_end_state {
        if keyboard.just_pressed(KeyCode::KeyM) {
            state.set(menu_state.menu_state.clone());
        } else if keyboard.just_pressed(KeyCode::KeyQ) {
            exit.write(AppExit::Success);
        }
    }
}
