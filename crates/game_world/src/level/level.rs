use bevy::prelude::*;
use game_core::states::AppState;
use game_core::tiled::{TiledMapBundle, TiledMapHandle};

pub struct WorldLevelModule;

impl Plugin for WorldLevelModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), setup);
    }
}

#[coverage(off)]
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let map_handle = TiledMapHandle(asset_server.load("maps/map.tmx"));
    commands.spawn((
        Name::new("Level"),
        TiledMapBundle {
            tiled_map: map_handle,
            ..default()
        }
    ));
}