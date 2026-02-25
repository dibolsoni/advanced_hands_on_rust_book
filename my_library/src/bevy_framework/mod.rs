use bevy::prelude::{Commands, Component, Entity, Query, With};

mod game_menu;
pub mod game_state;
mod phase;

pub use game_menu::*;
pub use game_state::*;
pub use phase::*;

pub fn cleanup<T>(query: Query<Entity, With<T>>, mut commands: Commands)
where
    T: Component,
{
    query
        .iter()
        .for_each(|entity| commands.entity(entity).despawn());
}
