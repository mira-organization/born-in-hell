#![feature(coverage_attribute)]

pub mod config;
pub mod key_converter;
pub mod states;
pub mod debug;
pub mod camera;
pub mod player;
pub mod tiled;

use bevy::prelude::*;
use crate::player::PlayerModule;
use crate::tiled::TiledModule;

/// Core of all game relevant resources and structures. This Plugin initializes resources
/// with `init_resource` from bevy. This Plugin is registered at [`ManagerPlugin`] which is
/// a part of the main.rs file.
pub struct GameCorePlugin;

impl Plugin for GameCorePlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerModule, TiledModule));
    }

}