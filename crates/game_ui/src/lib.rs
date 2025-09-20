#![feature(coverage_attribute)]

use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {

    #[coverage(off)]
    fn build(&self, _app: &mut App) { }

}