mod init;

use bevy::prelude::*;
use crate::player::init::PlayerInitService;

pub struct PlayerServiceImpl;

impl Plugin for PlayerServiceImpl {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerInitService);
    }
}