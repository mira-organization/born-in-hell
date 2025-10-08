use std::collections::HashMap;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use game_core::animation::{Animation, Animator};
use game_core::player::{Player, PlayerBody};
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::tiled::properties::PropertyValueExt;
use game_core::world::tiled_to_world_position;


pub struct PlayerInitService;

impl Plugin for PlayerInitService {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), init_player_loader);
    }
}

/// Registers the player initialization system for Tiled object layers.
/// Associates the `"Entities"` layer with the `init_player` system so it
/// can be invoked after a map is loaded and parsed.
///
/// # Parameters
/// * `object_layers` - Registry of object layers and their loader systems.
/// * `commands` - Used to register the `init_player` system.
#[coverage(off)]
fn init_player_loader(
    mut object_layers: ResMut<ObjectLayers>,
    mut commands: Commands
) {
    object_layers.loader_systems.insert(String::from("Entities"), commands.register_system(init_player));
}

/// Spawns the player entity from Tiled map data.
/// Looks up the `"Player"` object in the `"Entities"` layer, converts the
/// Tiled position to world space, builds a texture atlas layout, configures
/// animations (`idle`, `run`, `jump`), sets up physics (capsule collider,
/// kinematic character controller), and applies optional properties
/// (`health`, `base_health`) from the Tiled object.
///
/// # Parameters
/// * `commands` - Spawns the player, tiles, and related components.
/// * `object_layers` - Access to parsed Tiled object-layer data.
/// * `texture_atlas_layouts` - Asset storage to create/hold atlas layouts.
/// * `level_data` - Provides the loaded Tiled map for coordinate conversion.
/// * `asset_server` - Loads the player sprite texture.
#[coverage(off)]
fn init_player(
    mut commands: Commands,
    object_layers: Res<ObjectLayers>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level_data: Res<LevelData>,
    asset_server: Res<AssetServer>,
) {
    if let Some(object) = object_layers.get_data("Entities", "Player") {
        let map = level_data.map.as_ref().unwrap();

        let player_size = Vec2::new(13.0, 38.0);

        let position = tiled_to_world_position(Vec2::new(object.x, object.y), map) + player_size / 2.0;
        let frame_count = 19;
        let frame_size = UVec2::new(24, 38);

        let layout = TextureAtlasLayout::from_grid(
            frame_size,
            frame_count,
            1,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let mut animations = HashMap::new();

        animations.insert("idle".to_string(), Animation {
            start: 1,
            end: 8,
            frame_duration: 0.1,
            looping: true,
        });

        animations.insert("run".to_string(), Animation {
            start: 9,
            end: 14,
            frame_duration: 0.1,
            looping: true,
        });

        animations.insert("jump".to_string(), Animation {
            start: 16,
            end: 19,
            frame_duration: 0.1,
            looping: false,
        });


        let height = player_size.y;
        let width = player_size.x;
        let radius = width * 0.5;
        let half_height = (height * 0.5) - radius;

        let mut player = Player {
            body: PlayerBody {
                horizontal: 0,
                half_size: player_size / 2.0,
            },
            ..default()
        };

        if !object.properties.is_empty() {
            for (prop_name, prop_value) in object.properties.iter() {
                if prop_name.eq("health") {
                    player.stats.health = prop_value.i32_or(100);
                }

                if prop_name.eq("base_health") {
                    player.base_stats.health = prop_value.i32_or(100);
                }
            }
        }

        commands.spawn((
            Name::new("Player"),
            Transform::from_translation(Vec3::new(position.x, position.y, 10.)).with_scale(Vec3::splat(3.5)),
            GlobalTransform::IDENTITY,
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            Sprite {
                image: asset_server.load("sprites/player.png"),
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 0,
                }),
                ..Default::default()
            },
            Animator {
                animation: "idle".to_string(),
                animations,
                ..Default::default()
            },
            player,
            RigidBody::KinematicPositionBased,
            Collider::capsule_y(half_height.max(1.0), radius.max(1.0)),
            KinematicCharacterController {
                up: Vec2::Y,
                offset: CharacterLength::Absolute(0.02),
                slide: true,
                snap_to_ground: Some(CharacterLength::Absolute(4.0)),
                autostep: Some(CharacterAutostep {
                    max_height: CharacterLength::Absolute(6.0),
                    min_width: CharacterLength::Absolute(8.0),
                    include_dynamic_bodies: false
                }),
                max_slope_climb_angle: 55f32.to_radians(),
                min_slope_slide_angle: 65f32.to_radians(),
                ..default()
            },
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::ALL, Group::ALL)
        ));
    } else {
        error!("Player Data not found");
    }
}
