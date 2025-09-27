#![coverage(off)]

use bevy::prelude::*;

/// Marker component for the dedicated UI camera used to render HUD and
/// screen-space interfaces. Helps systems distinguish UI from world cameras.
#[derive(Component)]
pub struct CameraUi;

/// Marker component for the main in-game world camera. Used to target
/// world-space rendering and camera-control systems.
#[derive(Component)]
pub struct CameraGame;
