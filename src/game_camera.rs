use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use game_core::camera::{CameraGame, CameraUi};
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::LevelData;

const PIXEL_RATIO: f32 = 4.0;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), setup_ui_camera);
        app.add_systems(OnEnter(AppState::Preload), setup_game_camera)
            .add_systems(Update, update_game_camera.run_if(in_state(AppState::Preload)));
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
        Transform::IDENTITY,
        Msaa::Sample4,
        RenderLayers::from_layers(&[0, 1]),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0 / PIXEL_RATIO,
            far: -1000.0,
            near: 1000.0,
            ..OrthographicProjection::default_2d()
        }),
        CameraGame
    ));
}

#[coverage(off)]
fn update_game_camera(
    mut camera_query: Query<(&CameraGame, &Camera, &mut Transform), Without<Player>>,
    player_query: Query<(&Player, &Transform), Without<CameraGame>>,
    level_data: Res<LevelData>
) {
    if let Ok((_, camera, mut transform)) = camera_query.single_mut() {
        if let Ok((_, player_transform)) = player_query.single() {
            transform.translation.x = player_transform.translation.x;
            transform.translation.y = player_transform.translation.y;
        }

        if let Some(map) = level_data.map.as_ref() {
            if let Some(size) = camera.physical_viewport_size() {
                let map_width = (map.width * map.tile_width) as f32;
                let map_height = (map.height * map.tile_height) as f32;

                let view_half_size = Vec2::new(size.x as f32, size.y as f32) / (2.0 * PIXEL_RATIO);

                if transform.translation.x - view_half_size.x < 0.0 {
                    transform.translation.x = view_half_size.x;
                }

                if transform.translation.x + view_half_size.x > map_width {
                    transform.translation.x = map_width - view_half_size.x;
                }

                if transform.translation.y + view_half_size.y > map_height {
                    transform.translation.y = map_height - view_half_size.y;
                }

                if transform.translation.y - view_half_size.y < 0.0 {
                    transform.translation.y = view_half_size.y;
                }
            }
        }
    }
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
