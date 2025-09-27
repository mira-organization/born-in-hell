mod init;
mod input;

use bevy::prelude::*;
use crate::player::init::PlayerInitService;
use crate::player::input::PlayerInputService;

pub struct PlayerServiceImpl;

impl Plugin for PlayerServiceImpl {
    
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins((PlayerInitService, PlayerInputService));
    }
}