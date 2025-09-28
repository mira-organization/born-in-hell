#![coverage(off)]

use bevy::prelude::*;

/// Marker component designating an entity as a door trigger sensor.
/// Used to detect player overlap and emit door-related events.
#[derive(Component)]
pub struct DoorSensor;

/// Event fired when the player enters a door sensor area.
/// Consumed by systems that handle scene/level transitions or interactions.
#[derive(Event)]
pub struct DoorEntered;

/// Resource tracking whether the player is currently overlapping a door
/// sensor area. Useful for context-sensitive prompts (e.g., "Press E") and
/// debouncing interactions.
#[derive(Resource, Default)]
pub struct DoorOverlap {
    /// `true` when the player is inside any door sensor volume.
    pub inside: bool
}
