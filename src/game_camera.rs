use bevy::prelude::*;
use bevy::render::view::RenderLayers;
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
        RenderLayers::from_layers(&[0, 1]),
        Transform {
            translation: Vec3::new(5.0, 35.0, 55.0),
            rotation: Quat::from_rotation_x(-35.0_f32.to_radians()),
            ..Default::default()
        }
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
        Msaa::Sample4
    ));
}
