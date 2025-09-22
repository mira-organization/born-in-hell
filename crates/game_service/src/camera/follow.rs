use bevy::prelude::*;
use game_core::camera::CameraWorld;
use game_core::player::Player;
use game_core::states::{AppState, InGameStates};

pub struct CameraFollowService;

impl Plugin for CameraFollowService {
        #[coverage(off)]
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(AppState::InGame(InGameStates::Game)), follow_player_at_init);
            app.add_systems(Update, follow_player_update);
        }
}

fn follow_player_at_init(
    mut camera_query: Query<(&Transform, &mut CameraWorld), (With<CameraWorld>, Without<Player>)>,
    player_query: Query<(Entity, &Transform), With<Player>>
) {
    let (cam_transform, mut cam) = match camera_query.iter_mut().next() {
        Some(v) => v,
        None => return,
    };
    let (player_entity, player_transform) = match player_query.iter().next() {
        Some(v) => v,
        None => return,
    };

    let target = player_transform.translation + Vec3::new(0.0, 0.9, 0.0);

    let mut dir = cam_transform.translation - target;
    if dir.length_squared() < f32::EPSILON || dir.y <= 0.0 {
        dir = Vec3::new(0.0, 0.4, 1.0);
    }
    let dir = dir.normalize();

    cam.offset = dir * 8.0;
    cam.target = Some(player_entity);
    cam.stiffness = 8.0;
    cam.look_at = true;

    debug!("Follow player at init (offset: {:?})", cam.offset);
}

fn follow_player_update(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CameraWorld), With<CameraWorld>>,
    target_query: Query<&Transform, (Without<CameraWorld>, With<Player>)>,
) {
    let Some(player_tf) = target_query.iter().next() else { return; };
    let look_target = player_tf.translation + Vec3::new(0.0, 0.9, 0.0);

    for (mut cam_tf, cam) in camera_query.iter_mut() {
        let wanted = look_target + cam.offset;

        let alpha = 1.0 - (-cam.stiffness * time.delta_secs()).exp();
        cam_tf.translation = cam_tf.translation.lerp(wanted, alpha);

        if cam.look_at {
            cam_tf.look_at(look_target, Vec3::Y);
        }
    }
}