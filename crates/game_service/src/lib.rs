#![feature(coverage_attribute)]

mod player;

use bevy::prelude::*;
use crate::player::PlayerServiceImpl;

pub struct GameServicePlugin;

impl Plugin for GameServicePlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerServiceImpl);
    }

}