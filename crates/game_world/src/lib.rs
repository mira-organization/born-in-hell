#![feature(coverage_attribute)]

mod level;
mod collider;
mod objects;

use bevy::prelude::*;
use crate::collider::WorldColliderPlugin;
use crate::level::WorldLevelPlugin;
use crate::objects::WorldObjectsPlugin;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WorldLevelPlugin,
            WorldColliderPlugin,
            WorldObjectsPlugin
        ));
    }
    
}