use bevy::prelude::*;

#[derive(Component)]
pub struct DoorSensor;

#[derive(Event)]
pub struct DoorEntered;