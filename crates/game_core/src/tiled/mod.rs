#![coverage(off)]

use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind};
use std::path::Path;
use std::sync::Arc;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::TilemapBundle;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use thiserror::Error;
use tiled::{DefaultResourceCache, ObjectData};

pub struct TiledModule;

impl Plugin for TiledModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.register_asset_loader(TiledLoader);
        app.init_asset::<TiledMap>();
        app.init_resource::<LevelData>();
        app.init_resource::<ObjectLayers>();
        app.add_systems(Update, process_maps);
    }
}

#[derive(Resource, Default)]
pub struct LevelData {
    pub map: Option<tiled::Map>,
    pub collision_map: Vec<i32>,
    pub image_layers: Vec<ImageLayerData>,
}

#[derive(Clone)]
pub struct ImageLayerData {
    pub name: String,
    pub texture: Handle<Image>,
    pub color: Color,
    pub transform: Transform,
}

#[derive(Resource, Default)]
pub struct ObjectLayers {
    pub layer_data: HashMap<String, Vec<ObjectData>>,
    pub loader_systems: HashMap<String, SystemId>
}

#[derive(TypePath, Asset)]
pub struct TiledMap {
    pub map: tiled::Map,
    pub tilemap_textures: HashMap<usize, TilemapTexture>
}

struct BytesResourceReader {
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    type Resource = Cursor<Arc<[u8]>>;
    type Error = std::io::Error;

    fn read_from(&mut self, _path: &Path) -> Result<Self::Resource, Self::Error> {
        Ok(Cursor::new(self.bytes.clone()))
    }

}

#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    #[error("Tiled asset loading error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes: Vec<u8> = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let mut loader: tiled::Loader<DefaultResourceCache, _> = tiled::Loader::with_cache_and_reader(
            DefaultResourceCache::new(),
            BytesResourceReader::new(&bytes)
        );

        let map: tiled::Map = loader.load_tmx_map(load_context.path()).map_err(|error| {
            Error::new(ErrorKind::Other, format!("Could not load TMX map: {error}"))
        })?;

        let mut tilemap_textures = HashMap::new();

        for(tileset_index, tileset) in map.tilesets().iter().enumerate() {
            let tilemap_texture: TilemapTexture = match &tileset.image {
                None => {
                    warn!("Unsupported tileset type {}", tileset.name);
                    continue;
                },
                Some(img) => {
                    let texture_path: &str = img.source.to_str().unwrap();
                    let texture: Handle<Image> = load_context.load(texture_path);

                    TilemapTexture::Single(texture.clone())
                }
            };

            tilemap_textures.insert(tileset_index, tilemap_texture);
        }

        let asset_map: TiledMap = TiledMap {
            map,
            tilemap_textures
        };

        debug!("Loaded TMX map: {}", load_context.path().display());
        Ok(asset_map)
    }
}

#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Component, Default)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

#[derive(Component, Default)]
pub struct TiledMapLoaded(pub bool);

#[derive(Bundle, Default)]
pub struct TiledMapBundle {
    pub tiled_map: TiledMapHandle,
    pub storage: TiledLayersStorage,
    pub load_state: TiledMapLoaded,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings
}

#[coverage(off)]
fn process_maps(
    mut commands: Commands,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &mut TileStorage)>,
    mut map_query: Query<(&TiledMapHandle, &mut TiledMapLoaded, &mut TiledLayersStorage, &TilemapRenderSettings)>,
    mut object_layers: ResMut<ObjectLayers>,
    mut level_data: ResMut<LevelData>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((map_handle, mut load_state, mut layer_storage, render_settings))
        = map_query.single_mut() {
        if load_state.0 {
            return;
        }

        if let Some(tiled_map) = maps.get(&map_handle.0) {
            level_data.map = Some(tiled_map.map.clone());
            level_data.image_layers.clear();
            load_state.0 = true;

            for layer_entity in layer_storage.storage.values() {
                if let Ok((_, layer_title_storage)) = tile_storage_query.get(*layer_entity) {
                    for tile in layer_title_storage.iter().flatten() {
                        commands.entity(*tile).despawn();
                    }
                }
                commands.entity(*layer_entity).despawn();
            }

            for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                let tilemap_texture = tiled_map.tilemap_textures.get(&tileset_index).unwrap();

                let tile_size = TilemapTileSize {
                    x: tileset.tile_width as f32,
                    y: tileset.tile_height as f32
                };

                let tile_spacing = TilemapSpacing {
                    x: tileset.spacing as f32,
                    y: tileset.spacing as f32
                };

                for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                    let offset_x = layer.offset_x;
                    let offset_y = layer.offset_y;

                    match layer.layer_type() {
                        // Object Layer
                        tiled::LayerType::Objects(object_layer) => {
                            let data: Vec<ObjectData> = object_layer.object_data().iter().cloned().collect();
                            object_layers.layer_data.insert(layer.name.clone(), data);
                            if object_layers.loader_systems.contains_key(&layer.name) {
                                let system = object_layers.loader_systems[&layer.name];
                                commands.run_system(system);
                                debug!("Loaded system for layer {}", layer.name);
                            } else {
                                warn!("No System fond for ( {:?} )", layer.name);
                            }
                        }
                        // Background Layer
                        tiled::LayerType::Image(image_layer) => {
                            if let Some(img) = &image_layer.image {
                                if let Some(path) = img.source.to_str() {
                                    let texture: Handle<Image> = asset_server.load(path);

                                    let opacity = layer.opacity;
                                    let mut color = Color::WHITE.with_alpha(opacity);
                                    if let Some(tint) = layer.tint_color {
                                        let a = tint.alpha; let r = tint.red; let g = tint.green; let b = tint.blue;
                                        color = Color::srgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0,
                                                             (a as f32 / 255.0) * opacity);
                                    }

                                    let map_px_w = tiled_map.map.width  as f32 * tiled_map.map.tile_width  as f32;
                                    let map_px_h = tiled_map.map.height as f32 * tiled_map.map.tile_height as f32;

                                    let iw = img.width  as f32;
                                    let ih = img.height as f32;

                                    let (iw, ih) = if iw > 0.0 && ih > 0.0 {
                                        (iw, ih)
                                    } else {
                                        (0.0, 0.0)
                                    };

                                    let parent = commands.spawn((
                                        Name::new(format!("ImageLayer: {}", layer.name)),
                                        Sprite {
                                            anchor: Anchor::BottomLeft,
                                            image: texture.clone(),
                                            color,
                                            ..Default::default()
                                        },
                                        Transform::from_xyz(
                                            layer.offset_x,
                                            layer.offset_y,
                                            layer_index as f32,
                                        ),
                                        GlobalTransform::IDENTITY,
                                        Visibility::Visible,
                                        InheritedVisibility::VISIBLE,
                                    ))
                                        .id();

                                    if (image_layer.repeat_x || image_layer.repeat_y) && iw > 0.0 && ih > 0.0 {
                                        let tiles_x = if image_layer.repeat_x {
                                            ((map_px_w - layer.offset_x).max(0.0) / iw).ceil().max(1.0) as u32
                                        } else { 1 };

                                        let tiles_y = if image_layer.repeat_y {
                                            ((map_px_h - layer.offset_y).max(0.0) / ih).ceil().max(1.0) as u32
                                        } else { 1 };

                                        for iy in 0..tiles_y {
                                            for ix in 0..tiles_x {
                                                if ix == 0 && iy == 0 { continue; }
                                                let dx = iw * ix as f32;
                                                let dy = ih * iy as f32;

                                                commands.spawn((
                                                    Name::new("ImageTile"),
                                                    Sprite {
                                                        anchor: Anchor::BottomLeft,
                                                        image: texture.clone(),
                                                        color,
                                                        ..Default::default()
                                                    },
                                                    Transform::from_xyz(
                                                        (layer.offset_x + dx) - layer.offset_x,
                                                        (layer.offset_y + dy) - layer.offset_y,
                                                        layer_index as f32,
                                                    ),
                                                    GlobalTransform::IDENTITY,
                                                    Visibility::Visible,
                                                    InheritedVisibility::VISIBLE,
                                                    ChildOf(parent)
                                                ));
                                            }
                                        }
                                    }

                                    layer_storage.storage.insert(layer_index as u32, parent);
                                } else {
                                    warn!("Image layer '{}' is not Supported.", layer.name);
                                }
                            } else {
                                warn!("Image layer '{}' has no image yet!.", layer.name);
                            }
                        }
                        // Tile Layer
                        tiled::LayerType::Tiles(tile_layer) => {
                            if let tiled::TileLayer::Finite(layer_data) = tile_layer {
                                let map_size = TilemapSize {
                                    x: tiled_map.map.width,
                                    y: tiled_map.map.height
                                };

                                if layer.name.eq(&"Collision") {
                                    level_data.collision_map = vec![0; map_size.x as usize * map_size.y as usize];
                                }

                                let grid_size = TilemapGridSize {
                                    x: tiled_map.map.tile_width as f32,
                                    y: tiled_map.map.tile_height as f32
                                };

                                let map_type = match tiled_map.map.orientation {
                                    tiled::Orientation::Hexagonal => {
                                        TilemapType::Hexagon(HexCoordSystem::Row)
                                    }
                                    tiled::Orientation::Isometric => {
                                        TilemapType::Isometric(IsoCoordSystem::Diamond)
                                    }
                                    tiled::Orientation::Staggered => {
                                        TilemapType::Isometric(IsoCoordSystem::Staggered)
                                    }
                                    tiled::Orientation::Orthogonal => {
                                        TilemapType::Square
                                    }
                                };

                                let mut tile_storage = TileStorage::empty(map_size);
                                let layer_entity = commands.spawn_empty().id();

                                for x in 0..map_size.x {
                                    for y in 0..map_size.y {
                                        let mapped_y = (tiled_map.map.height - 1 - y) as i32;
                                        let mapped_x = x as i32;

                                        let layer_tile = match layer_data.get_tile(mapped_x, mapped_y) {
                                            Some(tile) => tile,
                                            None => continue
                                        };

                                        if tileset_index != layer_tile.tileset_index() {
                                            continue;
                                        }

                                        let layer_tile_data = match layer_data.get_tile_data(mapped_x, mapped_y) {
                                            Some(tile_data) => tile_data,
                                            None => continue
                                        };

                                        let texture_index = match tilemap_texture {
                                            TilemapTexture::Single(_) => layer_tile.id(),
                                        };

                                        let tile_pos = TilePos { x, y };

                                        let tile_entity = commands.spawn(
                                            TileBundle {
                                                position: tile_pos,
                                                tilemap_id: TilemapId(layer_entity),
                                                texture_index: TileTextureIndex(texture_index),
                                                flip: TileFlip {
                                                    x: layer_tile_data.flip_h,
                                                    y: layer_tile_data.flip_v,
                                                    d: layer_tile_data.flip_d
                                                },
                                                ..default()
                                            }
                                        ).id();

                                       tile_storage.set(&tile_pos, tile_entity);

                                        if layer.name.eq(&"Collision") {
                                            level_data.collision_map[(mapped_x + mapped_y * map_size.x as i32) as usize] = 1;
                                        }
                                    }
                                }

                                commands.entity(layer_entity).insert(
                                    TilemapBundle {
                                        grid_size,
                                        size: map_size,
                                        storage: tile_storage,
                                        texture: tilemap_texture.clone(),
                                        tile_size,
                                        spacing: tile_spacing,
                                        anchor: TilemapAnchor::BottomLeft,
                                        transform: Transform::from_xyz(offset_x, -offset_y, layer_index as f32),
                                        map_type,
                                        render_settings: *render_settings,
                                        ..default()
                                    }
                                );

                                layer_storage.storage.insert(layer_index as u32, layer_entity);
                            }
                        }
                        _ => {
                            warn!("Unsupported layer type {}", layer.name);
                        }
                    }
                }
            }
        }
    }
}