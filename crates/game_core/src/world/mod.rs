use bevy::prelude::*;

pub fn tiled_to_world_position(tiled_pos: Vec2, tiled_map: &tiled::Map) -> Vec2 {
    Vec2::new(
        tiled_pos.x,
        (tiled_map.height * tiled_map.tile_height) as f32 - tiled_pos.y
    )
}