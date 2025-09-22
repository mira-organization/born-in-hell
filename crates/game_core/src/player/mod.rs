#![coverage(off)]

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct PlayerModule;

impl Plugin for PlayerModule {

    #[coverage(off)]
    fn build(&self, _app: &mut App) {

    }
}

#[derive(Component, Default)]
pub struct Player {
    pub vy: f32
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub name: Name,
    pub player: Player,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub lock_axes: LockedAxes,
    pub ccd: Ccd,
    pub friction: Friction,
    pub restitution: Restitution,
    pub kenney_character_controller: KinematicCharacterController
}

impl Default for PlayerBundle {

    #[coverage(off)]
    fn default() -> Self {
        Self {
            name: Name::new("Player"),
            rigid_body: RigidBody::KinematicPositionBased,
            collider: Collider::capsule(Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.0, 1.6, 0.0), 0.2),
            lock_axes: LockedAxes::ROTATION_LOCKED_Z | LockedAxes::ROTATION_LOCKED_X,
            ccd: Ccd::default(),
            friction: Friction { coefficient: 0.9, combine_rule: CoefficientCombineRule::Min },
            restitution: Restitution { coefficient: 0.0, combine_rule: CoefficientCombineRule::Min },
            kenney_character_controller: KinematicCharacterController {
                offset: CharacterLength::Absolute(0.01),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(0.4),
                    min_width: CharacterLength::Absolute(0.2),
                    include_dynamic_bodies: false }
                ),

                snap_to_ground: Some(CharacterLength::Absolute(0.2)),
                max_slope_climb_angle: 50.0_f32.to_radians(),
                min_slope_slide_angle: 60.0_f32.to_radians(),
                slide: true, up: Vec3::Y, ..default()
            },
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            global_transform: GlobalTransform::default(),
            player: Player::default(),
        }
    }
}