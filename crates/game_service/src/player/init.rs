use std::collections::HashMap;
use bevy::prelude::*;
use game_core::aabb::AABB;
use game_core::animation::{Animation, Animator};
use game_core::player::Player;
use game_core::states::AppState;
use game_core::tiled::{LevelData, ObjectLayers};
use game_core::world::tiled_to_world_position;

const GRAVITY : f32 = 0.2;
const TERMINAL_VELOCITY : f32 = 4.0;
const JUMP_TIME : f32 = 0.225;
const JUMP_FORCE : f32 = 3.0;
const SPEED : f32 = 1.5;

pub struct PlayerInitService;

impl Plugin for PlayerInitService {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Preload), init_player_loader)
            .add_systems(Update, (handle_player_input,update_player_animations.after(handle_player_input)).run_if(in_state(AppState::Preload)))
            .add_systems(FixedUpdate, (handle_collisions,update_physics.before(handle_collisions)).run_if(in_state(AppState::Preload)));
    }
}

fn init_player_loader(
    mut object_layers: ResMut<ObjectLayers>,
    mut commands: Commands
) {
    object_layers.loader_systems.insert(String::from("Player"), commands.register_system(init_player));
}

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
        Transform::from_translation(Vec3::new(position.x, position.y, 10.)),
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
fn handle_collisions(
    level_data : Res<LevelData>,
    mut player_query : Query<(&mut Transform, &mut Player)>,
) {
    for(mut transform, mut player) in player_query.iter_mut() {
        if let Some(map) = level_data.map.as_ref() {
            let mut player_aabb = AABB::new(transform.translation.xy(), player.half_size);
            let tile_size = map.tile_width as f32;

            let left_tile = f32::floor(player_aabb.left() / tile_size) as usize;
            let right_tile = f32::ceil(player_aabb.right() / tile_size) as usize;
            let top_tile = map.height as usize - f32::ceil(player_aabb.top() / tile_size) as usize;
            let bottom_tile = map.height as usize - f32::floor(player_aabb.bottom() / tile_size) as usize;

            player.grounded = false;
            for x in left_tile..right_tile {
                for y in top_tile..bottom_tile  {
                    let tile_bounds = &AABB::new(
                        Vec2::new(
                            x as f32 * tile_size,
                            (map.height as usize - 1 - y) as f32 * tile_size
                        ) + Vec2::new(tile_size,tile_size) / 2.0,
                        Vec2::new(tile_size,tile_size) / 2.0,
                    );

                    let depth = player_aabb.get_intersection_depth(tile_bounds);
                    let abs_depth = depth.abs();
                    if depth != Vec2::ZERO {
                        if level_data.collision_map.is_empty() { continue }

                        if level_data.collision_map[x + y * map.width as usize] == 1 {
                            if abs_depth.y < abs_depth.x {
                                transform.translation.y += depth.y;
                                if depth.y > 0. {
                                    player.grounded = true;
                                }
                            }
                            else {
                                transform.translation.x += depth.x;
                            }
                            player_aabb = AABB::new(transform.translation.xy(), player.half_size);
                        }
                    }
                }
            }

        }
    }
}