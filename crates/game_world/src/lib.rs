#![feature(coverage_attribute)]

mod test_room;

use bevy::prelude::*;
use crate::test_room::TestRoomPlugin;

pub struct GameWorldPlugin;

impl Plugin for GameWorldPlugin {
    
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(TestRoomPlugin);
    }
    
}