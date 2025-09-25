use std::collections::HashMap;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use tiled::{LayerType, ObjectShape, TileLayer};
use game_core::animation::{Animation, Animator};
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::world::tiled_to_world_position;

const GRAVITY : f32 = 300.0;
const JUMP_TIME : f32 = 0.3;
const JUMP_FORCE : f32 = 250.0;
const SPEED : f32 = 200.0;

#[derive(Resource, Default)]
struct CollisionBuilt(bool);

pub struct PlayerInitService;

impl Plugin for PlayerInitService {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionBuilt>();

        app.add_systems(OnEnter(AppState::Preload), init_player_loader)

            .add_systems(Update, (handle_player_input,update_player_animations.after(handle_player_input), build_tile_colliders_once)
                .run_if(in_state(AppState::Preload)))

            .add_systems(FixedUpdate, (handle_collisions,update_physics.before(handle_collisions))
                .run_if(in_state(AppState::Preload)));
    }
}

#[coverage(off)]
fn init_player_loader(
    mut object_layers: ResMut<ObjectLayers>,
    mut commands: Commands
) {
    object_layers.loader_systems.insert(String::from("Player"), commands.register_system(init_player));
    object_layers.loader_systems.insert(String::from("Interact"), commands.register_system(door_test));
}

#[coverage(off)]
fn door_test() {
    info!("Door test");
}

#[coverage(off)]
fn init_player(
    mut commands: Commands,
    object_layers: Res<ObjectLayers>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    level_data: Res<LevelData>,
    asset_server: Res<AssetServer>,
) {
    let object_data = object_layers.layer_data["Player"].clone();
    let object = &object_data[0];
    let map = level_data.map.as_ref().unwrap();

    let player_size = Vec2::new(13.0,38.0);

    let position = tiled_to_world_position(Vec2::new(object.x,object.y),map) + player_size / 2.0;
    let frame_count = 19;
    let frame_size = UVec2::new(24,38);

    let layout = TextureAtlasLayout::from_grid(
        frame_size,
        frame_count,
        1,
        None,
        None,
    );
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let mut animations = HashMap::new();

    animations.insert("idle".to_string(),Animation{
        start: 1,
        end : 8,
        frame_duration : 0.1,
        looping : true,
    });

    animations.insert("run".to_string(),Animation{
        start: 9,
        end : 14,
        frame_duration : 0.1,
        looping : true,
    });

    animations.insert("jump".to_string(),Animation {
        start: 16,
        end : 19,
        frame_duration : 0.1,
        looping : false,
    });


    let height = player_size.y;
    let width = player_size.x;
    let radius = width * 0.5;
    let half_height = (height * 0.5) - radius;

    commands.spawn((
        Transform::from_translation(Vec3::new(position.x, position.y, 10.)).with_scale(Vec3::splat(1.0)),
        GlobalTransform::IDENTITY,
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        Sprite {
            image: asset_server.load("sprites/player.png"),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..Default::default()
        },
        Animator {
            animation: "idle".to_string(),
            animations,
            ..Default::default()
        },
        Player {
            jump_time : JUMP_TIME,
            jump_timer : 0.0,
            jump_force : JUMP_FORCE,
            speed : SPEED,
            released_jump : false,
            horizontal : 0,
            grounded : false,
            velocity : Vec2::new(0.,-0.1),
            half_size : player_size / 2.0,
        },

        RigidBody::KinematicPositionBased,
        Collider::capsule_y(half_height.max(1.0), radius.max(1.0)),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.02),
            slide: true,
            snap_to_ground: Some(CharacterLength::Absolute(1.5)),
            filter_flags: QueryFilterFlags::EXCLUDE_SENSORS,
            ..default()
        }
    ));
}

#[coverage(off)]
fn update_player_animations(
    mut player_query : Query<(&mut Player,&mut Sprite,&mut Animator)>
) {
    if let Ok((player,mut sprite,mut animator)) = player_query.single_mut() {

        if player.horizontal > 0 {
            sprite.flip_x = false;
        }
        else if player.horizontal < 0 {
            sprite.flip_x = true;
        }

        if !player.grounded {
            animator.animation = "jump".to_string();
        }
        else if player.horizontal != 0 {
            animator.animation = "run".to_string();
        }
        else {
            animator.animation = "idle".to_string();
        }
    }
}

#[coverage(off)]
fn handle_player_input(
    input : Res<ButtonInput<KeyCode>>,
    mut player_query : Query<&mut Player>,
) {
    if let Ok(mut player) = player_query.single_mut() {
        player.horizontal = 0;
        if input.pressed(KeyCode::KeyA) {
            player.horizontal -= 1;
        }
        if input.pressed(KeyCode::KeyD) {
            player.horizontal += 1;
        }

        if input.just_pressed(KeyCode::Space) && player.grounded {
            player.jump_timer = player.jump_time;
        }

        if input.just_released(KeyCode::Space) {
            player.jump_timer = 0.;
        }
    }
}

#[coverage(off)]
fn update_physics(
    time : Res<Time<Fixed>>,
    mut player_query : Query<(&mut KinematicCharacterController, &mut Player)>,
) {
    for(mut kcc, mut player) in player_query.iter_mut() {
        player.velocity.x = player.horizontal as f32 * player.speed;

        if player.jump_timer > 0. && player.grounded {
            let jump_force = player.jump_force;
            player.grounded = false;
            player.velocity.y = jump_force;
        }

        if !player.grounded {
            player.velocity.y -= GRAVITY * time.delta_secs();
        }

        let max_fall = 1200.0;
        if player.velocity.y < -max_fall {
            player.velocity.y = -max_fall;
        }

        let motion = player.velocity * time.delta_secs();
        kcc.translation = Some(motion);
        player.jump_timer -= time.delta_secs();
        if player.jump_timer < 0.0 { player.jump_timer = 0.0; }
    }
}

#[coverage(off)]
fn handle_collisions(
    mut query: Query<(&KinematicCharacterControllerOutput, &mut Player)>,
) {
    for (kcc_out, mut player) in query.iter_mut() {
        let was_grounded = player.grounded;
        player.grounded = kcc_out.grounded;
        if player.grounded && player.velocity.y < 0. {
            player.velocity.y = 0.;
        }

        let _ = was_grounded;
    }
}

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
                                    info!("Rect");
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
                                    info!("Ellipse");
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
                                            info!("Polygon");
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
                                        info!("Polyline");
                                    }
                                }
                                _ => {}
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

fn centroid(pts: &[Vec2]) -> Vec2 {
    if pts.is_empty() { return Vec2::ZERO; }
    let sum = pts.iter().fold(Vec2::ZERO, |acc, p| acc + *p);
    sum / (pts.len() as f32)
}

fn world_center_for_rect(tx: i32, ty_inv: i32, w: f32, h: f32, ox: f32, oy: f32, tw: f32, th: f32, mh: i32) -> (f32, f32) {
    let x0 = tx as f32 * tw + ox + w * 0.5;
    let y0 = (mh as f32 - 1.0 - ty_inv as f32) * th + (th - (oy + h * 0.5));
    (x0, y0)
}

fn polygon_world_points(tx: i32, ty_inv: i32, pts: &[(f32, f32)], ox: f32, oy: f32, tw: f32, th: f32, mh: i32) -> Vec<Vec2> {
    let base_x = tx as f32 * tw + ox;
    let base_y = (mh as f32 - 1.0 - ty_inv as f32) * th + (th - oy);
    pts.iter().map(|(px, py)| Vec2::new(base_x + *px, base_y - *py)).collect()
}
