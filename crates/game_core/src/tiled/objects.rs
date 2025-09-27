#![coverage(off)]

use bevy::prelude::*;

#[derive(Component)]
pub struct DoorSensor;

#[derive(Event)]
pub struct DoorEntered;

#[derive(Resource, Default)]
pub struct DoorOverlap { pub inside: bool }