use bevy::pbr::{CascadeShadowConfig, CascadeShadowConfigBuilder};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{AsyncSceneCollider, ComputedColliderShape, TriMeshFlags};
use game_core::states::AppState;

pub struct TestRoomPlugin;

impl Plugin for TestRoomPlugin {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), (load_test_room, load_light_source));
    }
}

#[coverage(off)]
fn load_test_room(mut commands: Commands, asset_server: Res<AssetServer>, mut next_state: ResMut<NextState<AppState>>) {
    commands.spawn((
        Name::new("TestRoom"),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("maps/test_room.glb"))),
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        AsyncSceneCollider {
            shape: Some(ComputedColliderShape::TriMesh(TriMeshFlags::MERGE_DUPLICATE_VERTICES)),
            ..default()
        }
    ));

    next_state.set(AppState::PostLoad);
}

#[coverage(off)]
fn load_light_source(mut commands: Commands) {

    let config: CascadeShadowConfig = CascadeShadowConfigBuilder {
        maximum_distance: 100.0,
        num_cascades: 4,
        ..default()
    }.into();

    commands.spawn((
        Name::new("Sun Light"),
        DirectionalLight {
            illuminance: 1000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            ..default()
        },
        config,
        Transform::from_xyz(4.0, 200.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}