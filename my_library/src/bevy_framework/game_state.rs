use crate::bevy_assets::{exit_menu, run_loading_menu, setup_loading_menu};
use crate::bevy_framework::{
    cleanup, run, setup_menu, Impulse, MenuElement, MenuResource, PhysicsTick,
};
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

pub struct GameStatePlugin<T> {
    menu_state: T,
    game_start_state: T,
    game_end_state: T,
}

impl<T> GameStatePlugin<T>
where
    T: States + FromWorld + FreelyMutableState,
{
    #[allow(clippy::new_without_default)]
    pub fn new(menu_state: T, game_start_state: T, game_end_state: T) -> Self {
        Self {
            menu_state,
            game_start_state,
            game_end_state,
        } //<callout id="generic_state.assign_playing" />
    }
}

//START: build
impl<T> Plugin for GameStatePlugin<T>
where
    T: States + Copy + FromWorld + FreelyMutableState + Default,
{
    //START: run_loader
    fn build(&self, app: &mut App) {
        app.add_message::<PhysicsTick>();
        app.add_message::<Impulse>();
        app.init_state::<T>();

        let start = MenuResource {
            menu_state: self.menu_state,
            game_start_state: self.game_start_state,
            game_end_state: self.game_end_state,
        };
        app.insert_resource(start);

        app.add_systems(OnEnter(self.menu_state), setup_menu::<T>);
        app.add_systems(Update, run::<T>.run_if(in_state(self.menu_state)));
        app.add_systems(OnExit(self.menu_state), cleanup::<MenuElement>);

        app.add_systems(OnEnter(self.game_end_state), setup_menu::<T>);
        app.add_systems(Update, run::<T>.run_if(in_state(self.game_end_state)));
        app.add_systems(OnExit(self.game_end_state), cleanup::<MenuElement>);

        app.add_systems(OnEnter(T::default()), setup_loading_menu);
        app.add_systems(Update, run_loading_menu::<T>.run_if(in_state(T::default())));
        app.add_systems(OnExit(T::default()), exit_menu);
    }
    //END: run_loader
}
//END: build
