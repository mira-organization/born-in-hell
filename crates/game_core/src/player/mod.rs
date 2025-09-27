#![coverage(off)]

use bevy::prelude::*;

pub const GRAVITY : f32 = 300.0;

pub struct PlayerModule;

impl Plugin for PlayerModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>();
        app.register_type::<Player>();
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Player {
    pub physic: PlayerPhysic,
    pub body: PlayerBody,
    pub stats: PlayerStats,
    pub base_stats: PlayerBaseStats
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerBaseStats {
    pub health: i32,
}

impl Default for PlayerBaseStats {
    fn default() -> Self {
        Self {
            health: 100
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerStats {
    pub health: i32
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 100
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Default)]
pub struct PlayerBody {
    pub half_size: Vec2,
    pub horizontal: i32,
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerPhysic {
    pub speed: f32,
    pub jump_force: f32,
    pub velocity : Vec2,
    pub grounded : bool,
    pub released_jump : bool,
    pub jump_time : f32,
    pub jump_timer : f32,
}

impl Default for PlayerPhysic {
    fn default() -> Self {
        Self {
            speed: 200.0,
            jump_force: 250.0,
            velocity: Vec2::new(0., -0.1),
            grounded: false,
            released_jump: false,
            jump_time: 0.3,
            jump_timer: 0.0
        }
    }
}

impl Default for Player {
    fn default() -> Self {
        Self {
            physic: PlayerPhysic::default(),
            body: PlayerBody::default(),
            stats: PlayerStats::default(),
            base_stats: PlayerBaseStats::default()
        }
    }
}

#[derive(Resource,Default)]
pub struct PlayerState {
    pub spawned : bool,
}