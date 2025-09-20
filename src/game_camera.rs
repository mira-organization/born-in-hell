use bevy::prelude::*;
use bevy::render::camera::{Projection, OrthographicProjection, ScalingMode};
use game_core::states::AppState;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    
    #[coverage(off)]   
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), setup_ui_camera);
        app.add_systems(OnEnter(AppState::PostLoad), setup_game_camera);
    }
}

#[coverage(off)]
fn setup_game_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        Msaa::Sample4,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            scale: 1.0,
            ..OrthographicProjection::default_3d()
        }),
    ));
}

#[coverage(off)]
fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1,
            ..default()
        },
        Msaa::Sample4,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
    ));
}
