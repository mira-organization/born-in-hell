use bevy::prelude::*;
use bevy_rapier2d::dynamics::RigidBody;
use bevy_rapier2d::geometry::Collider;
use tiled::{LayerType, ObjectShape, TileLayer};
use game_core::states::AppState;
use game_core::tiled::LevelData;

#[derive(Resource, Default)]
struct CollisionBuilt(bool);

pub struct WorldColliderPlugin;

impl Plugin for WorldColliderPlugin {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionBuilt>();
        app.add_systems(Update,
                        build_tile_colliders_once
                            .run_if(in_state(AppState::Preload)));
    }
}

/// Builds static physics colliders from Tiled tile collision data once per map.
/// Iterates all tile layers, spawns shape colliders (rect/ellipse/polygon/polyline)
/// from tileset collision objects, and falls back to full-tile boxes on a
/// layer named `"Collision"`. Everything is parented under a `"CollisionWorld"` entity.
///
/// # Parameters
/// * `commands` - Spawns collider entities and parenting hierarchy.
/// * `built` - One-shot flag to prevent rebuilding.
/// * `level_data` - Provides the loaded Tiled map.
#[coverage(off)]
fn build_tile_colliders_once(
    mut commands: Commands,
    mut built: ResMut<CollisionBuilt>,
    level_data: Res<LevelData>,
) {
    if built.0 { return; }
    let Some(map) = level_data.map.as_ref() else { return; };
    built.0 = true;

    let tw = map.tile_width as f32;
    let th = map.tile_height as f32;
    let mw = map.width as i32;
    let mh = map.height as i32;

    let parent = commands.spawn((
        Name::new("CollisionWorld"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::IDENTITY,
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
    )).id();

    for layer in map.layers() {
        let LayerType::Tiles(tile_layer) = layer.layer_type() else { continue };
        let TileLayer::Finite(ld) = tile_layer else { continue };

        for x in 0..mw {
            for y in 0..mh {
                let tx = x;
                let ty_inv = mh - 1 - y;
                let Some(tile) = ld.get_tile(tx, ty_inv) else { continue };

                let ts_index = tile.tileset_index();
                let tileset = &map.tilesets()[ts_index];
                let id = tile.id();

                let mut spawned_any = false;

                if let Some(tile_ref) = tileset.get_tile(id) {
                    if let Some(ol) = tile_ref.collision.as_ref() {
                        for obj in ol.object_data() {
                            match &obj.shape {
                                ObjectShape::Rect { width, height } => {
                                    let (cx, cy) = world_center_for_rect(tx, ty_inv, *width, *height, obj.x, obj.y, tw, th, mh);
                                    commands.spawn((
                                        Name::new("TileRect"),
                                        RigidBody::Fixed,
                                        Collider::cuboid(*width * 0.5, *height * 0.5),
                                        Transform::from_xyz(cx, cy, 0.0),
                                        GlobalTransform::IDENTITY,
                                        Visibility::Visible,
                                        InheritedVisibility::VISIBLE,
                                        ChildOf(parent),
                                    ));
                                    spawned_any = true;
                                }
                                ObjectShape::Ellipse { width, height } => {
                                    let r = width.min(*height) * 0.5;
                                    let (cx, cy) = world_center_for_rect(tx, ty_inv, *width, *height, obj.x, obj.y, tw, th, mh);
                                    commands.spawn((
                                        Name::new("TileEllipse"),
                                        RigidBody::Fixed,
                                        Collider::ball(r),
                                        Transform::from_xyz(cx, cy, 0.0),
                                        GlobalTransform::IDENTITY,
                                        Visibility::Visible,
                                        InheritedVisibility::VISIBLE,
                                        ChildOf(parent),
                                    ));
                                    spawned_any = true;
                                }
                                ObjectShape::Polygon { points } => {
                                    let world = polygon_world_points(tx, ty_inv, points, obj.x, obj.y, tw, th, mh);
                                    if world.len() >= 3 {
                                        let center = centroid(&world);
                                        let local: Vec<Vec2> = world.iter().map(|p| *p - center).collect();
                                        if let Some(ch) = Collider::convex_hull(&local) {
                                            commands.spawn((
                                                Name::new("TilePoly"),
                                                RigidBody::Fixed,
                                                ch,
                                                Transform::from_xyz(center.x, center.y, 0.0),
                                                GlobalTransform::IDENTITY,
                                                Visibility::Visible,
                                                InheritedVisibility::VISIBLE,
                                                ChildOf(parent),
                                            ));
                                            spawned_any = true;
                                        }
                                    }
                                }
                                ObjectShape::Polyline { points } => {
                                    let world = polygon_world_points(tx, ty_inv, points, obj.x, obj.y, tw, th, mh);
                                    if world.len() >= 2 {
                                        let center = centroid(&world);
                                        let local: Vec<Vec2> = world.iter().map(|p| *p - center).collect();
                                        commands.spawn((
                                            Name::new("TilePolyline"),
                                            RigidBody::Fixed,
                                            Collider::polyline(local, None),
                                            Transform::from_xyz(center.x, center.y, 0.0),
                                            GlobalTransform::IDENTITY,
                                            Visibility::Visible,
                                            InheritedVisibility::VISIBLE,
                                            ChildOf(parent),
                                        ));
                                        spawned_any = true;
                                    }
                                }
                                _ => { warn!("Unhandled collision shape"); }
                            }
                        }
                    }
                }

                if !spawned_any && layer.name == "Collision" {
                    let cx = (x as f32 + 0.5) * tw;
                    let cy = (y as f32 + 0.5) * th;
                    commands.spawn((
                        Name::new("CollisionBox"),
                        RigidBody::Fixed,
                        Collider::cuboid(tw * 0.5, th * 0.5),
                        Transform::from_xyz(cx, cy, 0.0),
                        GlobalTransform::IDENTITY,
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ChildOf(parent),
                    ));
                }
            }
        }
    }
}

/// Computes the arithmetic centroid of a list of points.
///
/// # Parameters
/// * `pts` - Collection of world-space points.
#[coverage(off)]
fn centroid(pts: &[Vec2]) -> Vec2 {
    if pts.is_empty() { return Vec2::ZERO; }
    let sum = pts.iter().fold(Vec2::ZERO, |acc, p| acc + *p);
    sum / (pts.len() as f32)
}

/// Converts a Tiled rectangle object (offsets and local size inside a tile)
/// into a world-space center position for spawning a collider.
///
/// # Parameters
/// * `tx`, `ty_inv` - Tile coordinates (with Y inverted to match world origin).
/// * `w`, `h` - Rectangle size in pixels.
/// * `ox`, `oy` - Rectangle local offsets inside the tile (Tiled space).
/// * `tw`, `th` - Tile width/height in pixels.
/// * `mh` - Map height in tiles (for Y flip).
#[coverage(off)]
fn world_center_for_rect(tx: i32, ty_inv: i32, w: f32, h: f32, ox: f32, oy: f32, tw: f32, th: f32, mh: i32) -> (f32, f32) {
    let x0 = tx as f32 * tw + ox + w * 0.5;
    let y0 = (mh as f32 - 1.0 - ty_inv as f32) * th + (th - (oy + h * 0.5));
    (x0, y0)
}

/// Converts Tiled polygon/polyline points (local to a tile) into world-space
/// points, applying tile position, local offsets, and Y-axis flip.
///
/// # Parameters
/// * `tx`, `ty_inv` - Tile coordinates (with Y inverted to match world origin).
/// * `pts` - Local points from Tiled (pixels).
/// * `ox`, `oy` - Local offset inside the tile (pixels).
/// * `tw`, `th` - Tile width/height (pixels).
/// * `mh` - Map height in tiles (for Y flip).
#[coverage(off)]
fn polygon_world_points(tx: i32, ty_inv: i32, pts: &[(f32, f32)], ox: f32, oy: f32, tw: f32, th: f32, mh: i32) -> Vec<Vec2> {
    let base_x = tx as f32 * tw + ox;
    let base_y = (mh as f32 - 1.0 - ty_inv as f32) * th + (th - oy);
    pts.iter().map(|(px, py)| Vec2::new(base_x + *px, base_y - *py)).collect()
}