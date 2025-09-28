use bevy::prelude::*;
use bevy_rapier2d::control::{KinematicCharacterController, KinematicCharacterControllerOutput};
use game_core::animation::Animator;
use game_core::config::GlobalConfig;
use game_core::player::{Player, GRAVITY};
use game_core::states::AppState;

pub struct PlayerInputService;

impl Plugin for PlayerInputService {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            handle_player_input,
            update_player_animations
                .after(handle_player_input)
        )
            .run_if(in_state(AppState::Preload)))

            .add_systems(FixedUpdate, (handle_collisions,update_physics.before(handle_collisions))
                .run_if(in_state(AppState::Preload)));
    }
}

/// Updates the player's animation state and sprite orientation based on
/// horizontal input and grounded status. Chooses between `"idle"`, `"run"`,
/// and `"jump"` and flips the sprite on the X axis to match facing.
///
/// # Parameters
/// * `player_query` - Single-entity query yielding mutable `Player`, `Sprite`,
///   and `Animator` components.
#[coverage(off)]
fn update_player_animations(
    mut player_query : Query<(&mut Player,&mut Sprite,&mut Animator)>
) {
    if let Ok((player,mut sprite,mut animator)) = player_query.single_mut() {

        if player.body.horizontal > 0 {
            sprite.flip_x = false;
        }
        else if player.body.horizontal < 0 {
            sprite.flip_x = true;
        }

        if !player.physic.grounded {
            animator.animation = "jump".to_string();
        }
        else if player.body.horizontal != 0 {
            animator.animation = "run".to_string();
        }
        else {
            animator.animation = "idle".to_string();
        }
    }
}

/// Translates keyboard input into player intent (left/right/jump) and updates
/// the `Player` component accordingly. Starts a jump window by priming
/// `jump_timer` when the jump is pressed while grounded; cancels it on release.
///
/// # Parameters
/// * `input` - Key state resource used to read pressed/just-pressed/released.
/// * `player_query` - Single-entity mutable access to the `Player`.
/// * `global_config` - Provides key bindings via `InputConfig`.
#[coverage(off)]
fn handle_player_input(
    input : Res<ButtonInput<KeyCode>>,
    mut player_query : Query<&mut Player>,
    global_config: Res<GlobalConfig>
) {
    let left_key = global_config.input_config.get_move_left_key();
    let right_key = global_config.input_config.get_move_right_key();
    let jump_key = global_config.input_config.get_jump_key();

    if let Ok(mut player) = player_query.single_mut() {
        player.body.horizontal = 0;
        if input.pressed(left_key) {
            player.body.horizontal -= 1;
        }
        if input.pressed(right_key) {
            player.body.horizontal += 1;
        }

        if input.just_pressed(jump_key) && player.physic.grounded {
            player.physic.jump_timer = player.physic.jump_time;
        }

        if input.just_released(jump_key) {
            player.physic.jump_timer = 0.;
        }
    }
}

/// Applies simple character physics using a kinematic controller:
/// sets horizontal velocity from input, triggers jump impulse if the jump
/// window is active and grounded, integrates gravity with clamping, and
/// writes per-tick translation into the controller.
///
/// # Parameters
/// * `time` - Fixed timestep used for stable integration.
/// * `player_query` - Mutable access to `KinematicCharacterController` and `Player`.
#[coverage(off)]
fn update_physics(
    time : Res<Time<Fixed>>,
    mut player_query : Query<(&mut KinematicCharacterController, &mut Player)>,
) {
    for(mut kcc, mut player) in player_query.iter_mut() {
        player.physic.velocity.x = player.body.horizontal as f32 * player.physic.speed;

        if player.physic.jump_timer > 0. && player.physic.grounded {
            let jump_force = player.physic.jump_force;
            player.physic.grounded = false;
            player.physic.velocity.y = jump_force;
        }

        if !player.physic.grounded {
            player.physic.velocity.y -= GRAVITY * time.delta_secs();
        }

        let max_fall = 1200.0;
        if player.physic.velocity.y < -max_fall {
            player.physic.velocity.y = -max_fall;
        }

        let motion = player.physic.velocity * time.delta_secs();
        kcc.translation = Some(motion);
        player.physic.jump_timer -= time.delta_secs();
        if player.physic.jump_timer < 0.0 { player.physic.jump_timer = 0.0; }
    }
}

/// Consumes kinematic controller output to update the grounded state and zero out
/// downward velocity on landing. Used to reconcile simulation after movement.
///
/// # Parameters
/// * `query` - Access to controller output paired with mutable `Player`.
#[coverage(off)]
fn handle_collisions(
    mut query: Query<(&KinematicCharacterControllerOutput, &mut Player)>,
) {
    for (kcc_out, mut player) in query.iter_mut() {
        let was_grounded = player.physic.grounded;
        player.physic.grounded = kcc_out.grounded;
        if player.physic.grounded && player.physic.velocity.y < 0. {
            player.physic.velocity.y = 0.;
        }

        let _ = was_grounded;
    }
}