use bevy::prelude::*;
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::world::tiled_to_world_position;

pub struct PlayerInitService;

impl Plugin for PlayerInitService {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), init_player_loader);
    }
}

fn init_player_loader(
    mut object_layers: ResMut<ObjectLayers>,
    mut commands: Commands
) {
    object_layers.loader_systems.insert(String::from("Player"), commands.register_system(init_player));
}

fn init_player(
    mut commands: Commands,
    object_layers: Res<ObjectLayers>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level_data: Res<LevelData>,
    asset_server: Res<AssetServer>,
) {
    let object_data = object_layers.layer_data["Player"].clone();
    let object = &object_data[0];
    let map = level_data.map.as_ref().unwrap();

    let player_size: Vec2 = Vec2::new(12.0, 38.0);

    let position = tiled_to_world_position(Vec2::new(object.x, object.y), map);
    let frame_count: u32 = 19;
    let frame_size: UVec2 = UVec2::new(24, 38);

    let layout = TextureAtlasLayout::from_grid(
        frame_size,
        frame_count,
        1,
        None,
        None
    );

    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn((
        Name::new("Player"),
        Transform::from_translation(position.extend(10.0)),
        Sprite {
            image: asset_server.load("sprites/player.png"),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0
            }),
            ..default()
        },
        Player
    ));
}