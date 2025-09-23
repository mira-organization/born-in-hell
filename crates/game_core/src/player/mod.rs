#![coverage(off)]

use bevy::prelude::*;
pub struct PlayerModule;

impl Plugin for PlayerModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>();
    }
}

#[derive(Component, Default)]
pub struct Player {
    pub speed : f32,
    pub jump_force : f32,
    pub velocity : Vec2,
    pub half_size : Vec2,
    pub grounded : bool,
    pub horizontal : i32,
    pub released_jump : bool,
    pub jump_time : f32,
    pub jump_timer : f32,
}

#[derive(Resource,Default)]
pub struct PlayerState {
    pub spawned : bool,
}