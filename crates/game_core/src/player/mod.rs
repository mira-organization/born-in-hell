#![coverage(off)]

use bevy::prelude::*;

pub const GRAVITY : f32 = 1000.0;

pub struct PlayerModule;

impl Plugin for PlayerModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerState>();
        app.register_type::<Player>();
    }
}

/// High-level player component aggregating physical properties, body shape,
/// and stats. Acts as the canonical state for player-related systems.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Player {
    /// Current physics state and movement parameters.
    pub physic: PlayerPhysic,
    /// Body dimensions and lateral facing info.
    pub body: PlayerBody,
    /// Mutable (runtime) stats such as current health.
    pub stats: PlayerStats,
    /// Baseline stats used for initialization or resets.
    pub base_stats: PlayerBaseStats
}

/// Baseline player stats used as defaults or for recalculation/reset flows.
#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerBaseStats {
    /// Maximum or default health value at full condition.
    pub health: i32,
}

impl Default for PlayerBaseStats {
    fn default() -> Self {
        Self { health: 100 }
    }
}

/// Live (mutable) player stats that change during gameplay.
#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerStats {
    /// Current health value.
    pub health: i32
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { health: 100 }
    }
}

/// Body representation used for collisions and orientation.
#[derive(Component, Reflect, Debug, Clone, Default)]
pub struct PlayerBody {
    /// Half-size of the player's collision/body box in world units.
    pub half_size: Vec2,
    /// Horizontal facing or input direction (-1, 0, 1).
    pub horizontal: i32,
}

/// Physics and movement parameters/state for the player.
#[derive(Component, Reflect, Debug, Clone)]
pub struct PlayerPhysic {
    /// Ground (horizontal) movement speed in units per second.
    pub speed: f32,
    /// Impulse/initial velocity applied at jump start.
    pub jump_force: f32,
    /// Current velocity vector.
    pub velocity : Vec2,
    /// Whether the player is considered grounded.
    pub grounded : bool,
    /// Whether jump was released to allow variable-height jumps.
    pub released_jump : bool,
    /// Maximum sustained jump time (seconds) for variable jump height.
    pub jump_time : f32,
    /// Remaining a time window (seconds) to continue applying jump.
    pub jump_timer : f32,
}

impl Default for PlayerPhysic {
    fn default() -> Self {
        Self {
            speed: 200.0,
            jump_force: 420.0,
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

/// Global resource tracking player lifecycle flags used by setup/spawn systems.
#[derive(Resource, Default)]
pub struct PlayerState {
    /// Whether the player entity has been spawned into the world.
    pub spawned : bool,
}
