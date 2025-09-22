use bevy::prelude::*;

#[derive(Component)]
pub struct CameraUi;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct CameraWorld {
    pub target: Option<Entity>,
    pub offset: Vec3,
    pub stiffness: f32,
    pub look_at: bool,
}