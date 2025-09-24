use std::collections::HashMap;
use bevy::prelude::*;
use game_core::aabb::AABB;
use game_core::animation::{Animation, Animator};
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::world::tiled_to_world_position;

const GRAVITY : f32 = 0.35;
const TERMINAL_VELOCITY : f32 = 4.0;
const JUMP_TIME : f32 = 0.5;
const JUMP_FORCE : f32 = 7.0;
const SPEED : f32 = 2.35;
const SKIN: f32 = 0.001;

pub struct PlayerInitService;

impl Plugin for PlayerInitService {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), init_player_loader)
            .add_systems(Update, (handle_player_input,update_player_animations.after(handle_player_input)).run_if(in_state(AppState::Preload)))
            .add_systems(FixedUpdate, (handle_collisions,update_physics.before(handle_collisions)).run_if(in_state(AppState::Preload)));
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

    commands.spawn((
        Transform::from_translation(Vec3::new(position.x, position.y, 10.)).with_scale(Vec3::splat(1.0)),
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
    mut player_query : Query<(&mut Transform, &mut Player)>,
) {
    for(mut transform, mut player) in player_query.iter_mut() {
        player.velocity.x = player.horizontal as f32 * player.speed;


        if player.jump_timer > 0. {
            let jump_force = player.jump_force;
            player.velocity.y += jump_force;
        }
        else {
            player.velocity.y -= GRAVITY;
        }

        if player.velocity.y.abs() > TERMINAL_VELOCITY {
            player.velocity.y = TERMINAL_VELOCITY * f32::signum(player.velocity.y);
        }

        transform.translation.x += player.velocity.x;
        transform.translation.y += player.velocity.y;

        if player.grounded {
            player.velocity.y = 0.;
        }
        player.jump_timer -= time.delta_secs();
    }
}

#[coverage(off)]
fn handle_collisions(
    level_data: Res<LevelData>,
    time: Res<Time>,
    mut q: Query<(&mut Transform, &mut Player)>,
) {
    let Some(map) = level_data.map.as_ref() else { return };
    if level_data.collision_map.is_empty() { return }

    let tile_w = map.tile_width as f32;
    let tile_h = map.tile_height as f32;
    let map_w = map.width as usize;
    let map_h = map.height as usize;
    let dt = time.delta_secs();

    for (mut tf, mut player) in q.iter_mut() {
        let mut pos = tf.translation.xy();
        let mut vel = player.velocity;
        let size = player.half_size;

        pos.x += vel.x * dt;
        let mut aabb = AABB::new(pos, size);

        let mut left   = ((aabb.left()  ) / tile_w).floor() as isize;
        let mut right  = ((aabb.right() ) / tile_w).ceil()  as isize;
        let mut bottom = ((aabb.bottom()) / tile_h).floor() as isize;
        let mut top    = ((aabb.top()   ) / tile_h).ceil()  as isize;

        left   = left.max(0).min(map_w as isize - 1);
        right  = right.max(0).min(map_w as isize);
        bottom = bottom.max(0).min(map_h as isize - 1);
        top    = top.max(0).min(map_h as isize);

        let mut corr_x = 0.0;
        for x in left..right {
            for y in bottom..top {
                let idx_y = (map_h as isize - 1 - y) as usize;
                let idx = x as usize + idx_y * map_w;
                if level_data.collision_map[idx] != 1 { continue }

                let tile_center = Vec2::new(
                    (x as f32 + 0.5) * tile_w,
                    (y as f32 + 0.5) * tile_h,
                );
                let tile_aabb = AABB::new(tile_center, Vec2::new(tile_w, tile_h) * 0.5);

                let depth = aabb.get_intersection_depth(&tile_aabb);
                if depth == Vec2::ZERO { continue }

                if depth.x.abs() < depth.y.abs() {
                    corr_x += depth.x.signum() * (depth.x.abs() + SKIN);
                    aabb.center.x += depth.x.signum() * (depth.x.abs() + SKIN);
                }
            }
        }
        pos.x += corr_x;
        if corr_x != 0.0 { vel.x = 0.0; }

        pos.y += vel.y * dt;
        aabb = AABB::new(pos, size);

        let mut left   = ((aabb.left()  ) / tile_w).floor() as isize;
        let mut right  = ((aabb.right() ) / tile_w).ceil()  as isize;
        let mut bottom = ((aabb.bottom()) / tile_h).floor() as isize;
        let mut top    = ((aabb.top()   ) / tile_h).ceil()  as isize;

        left   = left.max(0).min(map_w as isize - 1);
        right  = right.max(0).min(map_w as isize);
        bottom = bottom.max(0).min(map_h as isize - 1);
        top    = top.max(0).min(map_h as isize);

        let mut corr_y = 0.0;
        let falling = vel.y <= 0.0;
        let mut grounded = false;

        for x in left..right {
            for y in bottom..top {
                let idx_y = (map_h as isize - 1 - y) as usize;
                let idx = x as usize + idx_y * map_w;
                if level_data.collision_map[idx] != 1 { continue }

                let tile_center = Vec2::new(
                    (x as f32 + 0.5) * tile_w,
                    (y as f32 + 0.5) * tile_h,
                );
                let tile_aabb = AABB::new(tile_center, Vec2::new(tile_w, tile_h) * 0.5);

                let depth = aabb.get_intersection_depth(&tile_aabb);
                if depth == Vec2::ZERO { continue }

                if depth.y.abs() <= depth.x.abs() {
                    let push = depth.y.signum() * (depth.y.abs() + SKIN);
                    corr_y += push;
                    aabb.center.y += push;
                    if falling && push > 0.0 { grounded = true; }
                }
            }
        }
        pos.y += corr_y;
        if corr_y != 0.0 { vel.y = 0.0; }

        tf.translation.x = pos.x;
        tf.translation.y = pos.y;
        player.velocity = vel;
        player.grounded = grounded;
    }
}