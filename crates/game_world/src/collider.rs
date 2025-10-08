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

    let world_parent = commands.spawn((
        Name::new("CollisionWorld"),
        Transform::IDENTITY,
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

                let (flip_h, flip_v, flip_d) = ld
                    .get_tile_data(tx, ty_inv)
                    .map(|td| (td.flip_h, td.flip_v, td.flip_d))
                    .unwrap_or((false, false, false));

                let world_cx = (x as f32 + 0.5) * tw;
                let world_cy = (y as f32 + 0.5) * th;

                let mut spawned_any = false;

                if let Some(tile_ref) = tileset.get_tile(id) {
                    if let Some(ol) = tile_ref.collision.as_ref() {
                        for obj in ol.object_data() {
                            match &obj.shape {
                                ObjectShape::Rect { width, height } => {
                                    let mut pts = rect_points_local(tw, th, obj.x, obj.y, *width, *height).to_vec();
                                    apply_tiled_flips(&mut pts, flip_h, flip_v, flip_d);
                                    rotate_points(&mut pts, obj.rotation);
                                    let center = centroid(&pts);
                                    let local: Vec<Vec2> = pts.into_iter().map(|p| p - center).collect();
                                    if let Some(ch) = Collider::convex_hull(&local) {
                                        commands.spawn((
                                            Name::new("TileRectPoly"),
                                            RigidBody::Fixed,
                                            ch,
                                            Transform::from_translation(Vec3::new(world_cx + center.x, world_cy + center.y, 0.0)),
                                            GlobalTransform::IDENTITY,
                                            Visibility::Visible,
                                            InheritedVisibility::VISIBLE,
                                            ChildOf(world_parent),
                                        ));
                                        spawned_any = true;
                                    }
                                }

                                ObjectShape::Ellipse { width, height } => {
                                    let r = width.min(*height) * 0.5;
                                    let cx = (obj.x + width * 0.5) - tw * 0.5;
                                    let cy = (th - (obj.y + height * 0.5)) - th * 0.5;
                                    commands.spawn((
                                        Name::new("TileEllipseBall"),
                                        RigidBody::Fixed,
                                        Collider::ball(r),
                                        Transform::from_translation(Vec3::new(world_cx + cx, world_cy + cy, 0.0)),
                                        GlobalTransform::IDENTITY,
                                        Visibility::Visible,
                                        InheritedVisibility::VISIBLE,
                                        ChildOf(world_parent),
                                    ));
                                    spawned_any = true;
                                }

                                ObjectShape::Polygon { points } => {
                                    let mut pts = raw_points_local(tw, th, obj.x, obj.y, points);
                                    apply_tiled_flips(&mut pts, flip_h, flip_v, flip_d);
                                    rotate_points(&mut pts, obj.rotation);
                                    if pts.len() >= 3 {
                                        let center = centroid(&pts);
                                        let local: Vec<Vec2> = pts.into_iter().map(|p| p - center).collect();
                                        if let Some(ch) = Collider::convex_hull(&local) {
                                            commands.spawn((
                                                Name::new("TilePoly"),
                                                RigidBody::Fixed,
                                                ch,
                                                Transform::from_translation(Vec3::new(world_cx + center.x, world_cy + center.y, 0.0)),
                                                GlobalTransform::IDENTITY,
                                                Visibility::Visible,
                                                InheritedVisibility::VISIBLE,
                                                ChildOf(world_parent),
                                            ));
                                            spawned_any = true;
                                        }
                                    }
                                }

                                ObjectShape::Polyline { points } => {
                                    let mut pts = raw_points_local(tw, th, obj.x, obj.y, points);
                                    apply_tiled_flips(&mut pts, flip_h, flip_v, flip_d);
                                    rotate_points(&mut pts, obj.rotation);
                                    if pts.len() >= 2 {
                                        let center = centroid(&pts);
                                        let local: Vec<Vec2> = pts.into_iter().map(|p| p - center).collect();
                                        commands.spawn((
                                            Name::new("TilePolyline"),
                                            RigidBody::Fixed,
                                            Collider::polyline(local, None),
                                            Transform::from_translation(Vec3::new(world_cx + center.x, world_cy + center.y, 0.0)),
                                            GlobalTransform::IDENTITY,
                                            Visibility::Visible,
                                            InheritedVisibility::VISIBLE,
                                            ChildOf(world_parent),
                                        ));
                                        spawned_any = true;
                                    }
                                }

                                _ => { warn!("Unhandled collision shape"); }
                            }
                        }
                    }
                }

                if !spawned_any && layer.name.starts_with("Collision") {
                    commands.spawn((
                        Name::new("CollisionBox"),
                        RigidBody::Fixed,
                        Collider::cuboid(tw * 0.5, th * 0.5),
                        Transform::from_xyz((x as f32 + 0.5) * tw, (y as f32 + 0.5) * th, 0.0),
                        GlobalTransform::IDENTITY,
                        Visibility::Visible,
                        InheritedVisibility::VISIBLE,
                        ChildOf(world_parent),
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

#[coverage(off)]
fn rect_points_local(tw: f32, th: f32, ox: f32, oy: f32, w: f32, h: f32) -> [Vec2; 4] {
    let pts = [
        (0.0,     0.0),
        (w,       0.0),
        (w,       h),
        (0.0,     h),
    ];
    let mut out = [Vec2::ZERO; 4];
    for (i, (px, py)) in pts.iter().enumerate() {
        let x = (ox + *px) - tw * 0.5;
        let y = (th - (oy + *py)) - th * 0.5;
        out[i] = Vec2::new(x, y);
    }
    out
}

#[coverage(off)]
fn raw_points_local(tw: f32, th: f32, ox: f32, oy: f32, pts: &[(f32,f32)]) -> Vec<Vec2> {
    pts.iter().map(|(px,py)| {
        let x = (ox + *px) - tw * 0.5;
        let y = (th - (oy + *py)) - th * 0.5;
        Vec2::new(x,y)
    }).collect()
}

#[coverage(off)]
fn apply_tiled_flips(pts: &mut [Vec2], flip_h: bool, flip_v: bool, flip_d: bool) {
    if flip_h {
        for p in pts.iter_mut() { p.x = -p.x; }
    }
    if flip_v {
        for p in pts.iter_mut() { p.y = -p.y; }
    }
    if flip_d {
        for p in pts.iter_mut() { std::mem::swap(&mut p.x, &mut p.y); }
    }
}

#[coverage(off)]
fn rotate_points(pts: &mut [Vec2], deg_clockwise: f32) {
    let a = -deg_clockwise.to_radians();
    let (s, c) = a.sin_cos();
    for p in pts.iter_mut() {
        let x = p.x * c - p.y * s;
        let y = p.x * s + p.y * c;
        p.x = x; p.y = y;
    }
}