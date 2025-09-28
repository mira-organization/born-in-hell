#![coverage(off)]

use std::collections::HashMap;
use bevy::prelude::*;

pub struct AnimationModule;

impl Plugin for AnimationModule {

    #[coverage(off)]   
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_animations);
    }
}

/// Component that manages sprite-sheet-based animations for an entity's `Sprite`.
/// Keeps track of the active and previous animation, a registry of available
/// animations, and a countdown timer used to advance frames.
#[derive(Component, Default)]
pub struct Animator {
    /// Name of the currently active animation. Must exist in `animations`.
    pub animation: String,
    /// Name of the animation that was active in the previous update tick.
    pub previous_animation: String,
    /// Registry of available animations addressed by name.
    pub animations: HashMap<String, Animation>,
    /// Remaining time (in seconds) until the next frame advance.
    pub timer: f32,
}

/// Describes a single animation sequence within a texture atlas by its frame
/// range, per-frame duration, and whether it loops when reaching the end.
#[derive(Clone, Default)]
pub struct Animation {
    /// Duration (in seconds) each frame is displayed.
    pub frame_duration: f32,
    /// Inclusive start frame index within the atlas (0-based in code usage).
    pub start: usize,
    /// Inclusive end frame index within the atlas.
    pub end: usize,
    /// Whether the animation restarts from `start` after the last frame.
    pub looping: bool,
}

/// Advances sprite animations over time and handles animation switching,
/// frame progression, and looping using the entity's `Animator` state and
/// the `Sprite` texture atlas index.
///
/// # Parameters
/// * `time` - Global time resource used to decrement animation timers.
/// * `animator_query` - Query over entities providing mutable access to
///   their `Animator` and `Sprite` to update timers and atlas indices.
#[coverage(off)]
fn update_animations(
    time : Res<Time>,
    mut animator_query : Query<(&mut Animator, &mut Sprite)>
) {
    for(mut animator, mut sprite) in animator_query.iter_mut() {
        animator.timer -= time.delta_secs();

        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            if animator.animation != animator.previous_animation {
                let animation = animator.animations[&animator.animation].clone();
                atlas.index = animation.start - 1;
                animator.timer = animation.frame_duration;
            }

            if animator.timer <= 0. {
                let animation = animator.animations[&animator.animation].clone();
                animator.timer = animation.frame_duration;
                atlas.index += 1;
                if atlas.index > animation.end - 1 {
                    if animation.looping {
                        atlas.index = animation.start - 1;
                    }
                    else {
                        atlas.index = animation.end - 1;
                    }
                }
            }
        }

        animator.previous_animation = animator.animation.clone();
    }
}