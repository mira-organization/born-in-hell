mod level;

use bevy::prelude::*;
use crate::level::level::WorldLevelModule;

pub struct WorldLevelPlugin;

impl Plugin for WorldLevelPlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldLevelModule);
    }
}