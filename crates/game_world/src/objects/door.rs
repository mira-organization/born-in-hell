use bevy::prelude::*;
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::{ActiveCollisionTypes, ActiveEvents, Collider, CollisionGroups, Group, Sensor};
use bevy_rapier2d::pipeline::CollisionEvent;
use game_core::config::GlobalConfig;
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::tiled::objects::{DoorEntered, DoorOverlap, DoorSensor};
use game_core::tiled::properties::ObjectShapeExt;
use game_core::world::tiled_to_world_position;

pub struct WorldDoorModule;

impl Plugin for WorldDoorModule {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), init_doors)
            .add_systems(Update, (door_observer, door_interact, on_door_entered)
                .run_if(in_state(AppState::Preload)));
    }
}

/// Registers the door loader system for the `"Interact"` Tiled object layer,
/// mapping it to the `door_creation` system for sensor instantiation.
///
/// # Parameters
/// * `object_layers` - Registry of object layers and their associated systems.
/// * `commands` - Used to register the `door_creation` system.
#[coverage(off)]
fn init_doors(
    mut object_layers: ResMut<ObjectLayers>,
    mut commands: Commands
) {
    object_layers.loader_systems.insert(String::from("Interact"), commands.register_system(door_creation));
}

/// Spawns a rectangular door sensor from a Tiled object named `"DoorTest"`
/// in the `"Interact"` layer when its `user_type` is `"observe"`.
/// Converts Tiled coordinates to world space and creates a fixed sensor collider.
///
/// # Parameters
/// * `commands` - Spawns the sensor entity and its components.
/// * `object_layers` - Access to parsed Tiled object data.
/// * `level_data` - Provides the loaded map for coordinate conversion.
#[coverage(off)]
fn door_creation(
    mut commands: Commands,
    object_layers: Res<ObjectLayers>,
    level_data: Res<LevelData>
) {
    let Some(map) = level_data.map.as_ref() else { return; };
    let Some(object) = object_layers.get_data("Interact", "DoorTest") else { return; };
    if !object.user_type.eq_ignore_ascii_case(&"observe") { return; }

    let width = object.shape.get_width();
    let height = object.shape.get_height();

    let origin = tiled_to_world_position(Vec2::new(object.x, object.y), map);
    let center = origin + Vec2::new(width * 0.5, height * 0.5);

    commands.spawn((
        Name::new("DoorSensor"),
        DoorSensor,
        Transform::from_xyz(center.x, center.y - height, 0.0),
        GlobalTransform::IDENTITY,
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        RigidBody::Fixed,
        Collider::cuboid(width * 0.5, height * 0.5),
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::all(),
        CollisionGroups::new(Group::ALL, Group::ALL)
    ));
}

/// Observes Rapier collision events to track player overlap with any
/// `DoorSensor`. Sets `DoorOverlap.inside` true on entering and false on exit.
///
/// # Parameters
/// * `ev` - Stream of collision start/stop events.
/// * `door_q` - Query to identify door sensor entities.
/// * `player_q` - Query to identify the player entity.
/// * `overlap` - Mutable state tracking whether the player is inside a door.
#[coverage(off)]
fn door_observer(
    mut ev: EventReader<CollisionEvent>,
    door_q: Query<Entity, With<DoorSensor>>,
    player_q: Query<Entity, With<Player>>,
    mut overlap: ResMut<DoorOverlap>,
) {
    for e in ev.read() {
        match e {
            CollisionEvent::Started(a, b, _) => {
                let a_is_door = door_q.get(*a).is_ok();
                let b_is_door = door_q.get(*b).is_ok();
                let a_is_player = player_q.get(*a).is_ok();
                let b_is_player = player_q.get(*b).is_ok();
                if (a_is_door && b_is_player) || (b_is_door && a_is_player) {
                    overlap.inside = true;
                }
            }
            CollisionEvent::Stopped(a, b, _) => {
                let a_is_door = door_q.get(*a).is_ok();
                let b_is_door = door_q.get(*b).is_ok();
                let a_is_player = player_q.get(*a).is_ok();
                let b_is_player = player_q.get(*b).is_ok();
                if (a_is_door && b_is_player) || (b_is_door && a_is_player) {
                    overlap.inside = false;
                }
            }
        }
    }
}

/// Handles the interacted input while overlapping a door sensor and emits
/// `DoorEntered` when the configured interacted key is just pressed.
///
/// # Parameters
/// * `input` - Keyboard input state.
/// * `overlap` - Read-only overlap state from the observer.
/// * `global_config` - Access to input bindings (interact key).
/// * `writer` - Event writer for `DoorEntered`.
#[coverage(off)]
fn door_interact(
    input: Res<ButtonInput<KeyCode>>,
    overlap: Res<DoorOverlap>,
    global_config: Res<GlobalConfig>,
    mut writer: EventWriter<DoorEntered>,
) {
    if !overlap.inside { return; }
    let interact_key = global_config.input_config.get_interact_key();
    if input.just_pressed(interact_key) {
        writer.write(DoorEntered);
    }
}

/// Consumes `DoorEntered` events to trigger door-entry side effects
/// (e.g., logging, transitions, scene loads).
///
/// # Parameters
/// * `ev` - Reader for door-entered events.
#[coverage(off)]
fn on_door_entered(mut ev: EventReader<DoorEntered>) {
    for _ in ev.read() {
        info!("Door Entered");
    }
}