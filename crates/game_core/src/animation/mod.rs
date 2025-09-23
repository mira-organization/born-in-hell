use std::collections::HashMap;
use bevy::prelude::*;

pub struct AnimationModule;

impl Plugin for AnimationModule {

    #[coverage(off)]   
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, update_animations);
    }
}

#[derive(Component,Default)]
pub struct Animator {
    pub animation : String,
    pub previous_animation : String,
    pub animations : HashMap<String, Animation>,
    pub timer : f32,
}

#[derive(Clone,Default)]
pub struct Animation {
    pub frame_duration : f32,
    pub start : usize,
    pub end : usize,
    pub looping : bool,
}

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