use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use game_core::player::{Player, PlayerBundle};
use game_core::states::{is_state_in_game, AppState, InGameStates};

pub struct PlayerInitService;

impl Plugin for PlayerInitService {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::PostLoad), create_player);
        app.add_systems(Update, player_gravity_and_move.run_if(is_state_in_game));
    }
}

fn create_player(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    commands.spawn((
        PlayerBundle::default(),
        SceneRoot(assets_server.load(GltfAssetLabel::Scene(0).from_asset("player/player.glb")))
    ));

    next_state.set(AppState::InGame(InGameStates::Game));
}

fn player_gravity_and_move(
    time: Res<Time>,
    mut q: Query<(
        &mut KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
        &mut Player,
    ), With<Player>>,
) {
    let dt = time.delta_secs();
    const G: f32 = -9.81;
    const GRAVITY_SCALE: f32 = 3.0;

    for (mut ctrl, out_opt, mut kin) in q.iter_mut() {
        let grounded = out_opt.map_or(false, |o| o.grounded);

        if grounded {
            if kin.vy < 0.0 { kin.vy = 0.0; }
        } else {
            kin.vy += G * GRAVITY_SCALE * dt;
            if kin.vy < -50.0 { kin.vy = -50.0; }
        }

        let horizontal = Vec3::ZERO;

        let move_delta = horizontal * dt + Vec3::Y * (kin.vy * dt);
        ctrl.translation = Some(move_delta);
    }
}