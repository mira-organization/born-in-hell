mod follow;

use bevy::prelude::*;
use crate::camera::follow::CameraFollowService;

pub struct CameraServiceImpl;

impl Plugin for CameraServiceImpl {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraFollowService);
    }
}