#![feature(coverage_attribute)]

mod level;

use bevy::prelude::*;
use crate::level::WorldLevelPlugin;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(
            WorldLevelPlugin
        );
    }
    
}