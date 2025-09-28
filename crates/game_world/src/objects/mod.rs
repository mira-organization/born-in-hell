mod door;

use bevy::prelude::*;

pub struct WorldObjectsPlugin;

impl Plugin for WorldObjectsPlugin {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(door::WorldDoorModule);
    }
}