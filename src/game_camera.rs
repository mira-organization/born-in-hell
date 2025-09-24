use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::render::view::RenderLayers;
use game_core::camera::{CameraGame, CameraUi};
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::LevelData;

const ZOOM: f32 = 2.0; // 4k = 8.0

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
        Transform::from_xyz(0.0, 0.0, 1000.0),
        Msaa::Sample4,
        RenderLayers::from_layers(&[0, 1]),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            near: -1000.0,
            far:  1000.0,
            ..OrthographicProjection::default_2d()
        }),
        CameraGame
    ));
}

#[coverage(off)]
fn update_game_camera(
    time: Res<Time>,
    mut q_cam: Query<(&Camera, &mut Transform, &mut Projection), (Without<Player>, With<CameraGame>)>,
    q_player: Query<&Transform, (Without<CameraGame>, With<Player>)>,
    level_data: Res<LevelData>,
) {
    let (camera, mut cam_tf, mut proj) = if let Ok(x) = q_cam.single_mut() { x } else { return };
    let player_tf = if let Ok(x) = q_player.single() { x } else { return };

    let follow_strength = 6.0;
    let dt = time.delta_secs();
    let t = 1.0 - (-follow_strength * dt).exp();
    let target = player_tf.translation.truncate();
    let cam_xy = cam_tf.translation.truncate();
    let new_xy = cam_xy + (target - cam_xy) * t;

    let (map, view) = if let (Some(m), Some(v)) = (level_data.map.as_ref(), camera.logical_viewport_size()) { (m, v) } else { return };
    let map_w = (map.width * map.tile_width) as f32;
    let map_h = (map.height * map.tile_height) as f32;
    let view_w = view.x;
    let view_h = view.y;

    if let Projection::Orthographic(ortho) = &mut *proj {
        let cover = (view_w / map_w).max(view_h / map_h);
        let base = 1.0 / cover;
        ortho.scale = base / ZOOM;

        let half = Vec2::new(view_w, view_h) * ortho.scale * 0.5;

        let mut p = new_xy;
        if map_w > half.x * 2.0 { p.x = p.x.clamp(half.x, map_w - half.x) } else { p.x = map_w * 0.5; }
        if map_h > half.y * 2.0 { p.y = p.y.clamp(half.y, map_h - half.y) } else { p.y = map_h * 0.5; }

        cam_tf.translation.x = p.x;
        cam_tf.translation.y = p.y;
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
