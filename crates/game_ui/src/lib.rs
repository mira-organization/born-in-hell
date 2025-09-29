#![feature(coverage_attribute)]

mod debug_overlay;

use bevy::prelude::*;
use crate::debug_overlay::DebugOverlayScreen;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(DebugOverlayScreen);
    }

}