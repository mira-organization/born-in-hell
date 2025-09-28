#![coverage(off)]

use bevy::prelude::*;

/// Converts a Tiled pixel position (origin at top-left) into a world position
/// with a bottom-left origin by flipping the Y axis against the map's pixel
/// height (`map.height * map.tile_height`).
///
/// # Parameters
/// * `tiled_pos` - Position in Tiled pixel coordinates (top-left origin).
/// * `tiled_map` - Map used to derive total pixel height for Y inversion.
#[coverage(off)]
pub fn tiled_to_world_position(tiled_pos: Vec2, tiled_map: &tiled::Map) -> Vec2 {
    Vec2::new(
        tiled_pos.x,
        (tiled_map.height * tiled_map.tile_height) as f32 - tiled_pos.y
    )
}
