use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use game_core::camera::{CameraGame, CameraUi};
use game_core::states::AppState;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), setup_ui_camera);
        app.add_systems(OnEnter(AppState::Preload), setup_game_camera);
    }
}

#[coverage(off)]
fn setup_game_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Msaa::Sample4,
        RenderLayers::from_layers(&[0, 1]),
        CameraGame
    ));
}

#[coverage(off)]
fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayers::layer(1),
        Msaa::Sample4,
        CameraUi
    ));
}
