#![feature(coverage_attribute)]

mod player;
mod camera;

use bevy::prelude::*;
use crate::camera::CameraServiceImpl;
use crate::player::PlayerServiceImpl;

pub struct GameServicePlugin;

impl Plugin for GameServicePlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerServiceImpl, CameraServiceImpl));
    }

}