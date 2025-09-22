#![feature(coverage_attribute)]

use bevy::prelude::*;
use bevy_ecs_tilemap::TilemapPlugin;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);
    }
    
}