#![coverage(off)]

use bevy::prelude::*;
pub struct PlayerModule;

impl Plugin for PlayerModule {

    #[coverage(off)]
    fn build(&self, _app: &mut App) {

    }
}

#[derive(Component)]
pub struct Player;