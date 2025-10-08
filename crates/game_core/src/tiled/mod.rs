#![coverage(off)]

pub mod properties;
pub mod objects;

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
use crate::tiled::objects::{DoorEntered, DoorOverlap};

pub struct TiledModule;

impl Plugin for TiledModule {

    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.register_asset_loader(TiledLoader);
        app.init_asset::<TiledMap>();
        app.init_resource::<LevelData>();
        app.init_resource::<ObjectLayers>();
        app.init_resource::<DoorOverlap>();
        app.add_event::<DoorEntered>();
        app.add_systems(Update, process_maps);
    }
}

/// Level container holding the parsed Tiled map, a precomputed collision
/// grid, and renderable image layers extracted from the map.
#[derive(Resource, Default)]
pub struct LevelData {
    /// The loaded Tiled map; `None` until parsing completes.
    pub map: Option<tiled::Map>,
    /// Flat collision mask (tile-aligned), typically 0/1 or similar flags.
    pub collision_map: Vec<i32>,
    /// Ordered a set of image layers with texture, tint, and transform.
    pub image_layers: Vec<ImageLayerData>,
}

/// Renderable description of a single Tiled image layer including texture,
/// color/tint, and world transform.
#[derive(Clone)]
pub struct ImageLayerData {
    /// Layer name as defined in Tiled.
    pub name: String,
    /// Handle to the Bevy image for this layer.
    pub texture: Handle<Image>,
    /// Multiplicative tint color (includes opacity).
    pub color: Color,
    /// Transform applied when drawing this layer.
    pub transform: Transform,
}

/// Registry of Tiled object layers keyed by layer name, plus optional loader
/// systems associated with those layers for dynamic instantiation.
#[derive(Resource, Default)]
pub struct ObjectLayers {
    /// Mapping: layer name -> list of parsed object records.
    pub layer_data: HashMap<String, Vec<ObjectData>>,
    /// Mapping: layer name -> system that knows how to spawn/process it.
    pub loader_systems: HashMap<String, SystemId>
}

impl ObjectLayers {
    /// Retrieves a cloned `ObjectData` by layer and object name.
    ///
    /// # Parameters
    /// * `layer_name` - Name of the Tiled object layer.
    /// * `key` - Object name to match within the layer.
    pub fn get_data(&self, layer_name: &str, key: &str) -> Option<ObjectData> {
        let data = None;
        if let Some(layer_data) = self.layer_data.get(layer_name) {
            for obj_data in layer_data {
                if obj_data.name.eq(key) {
                    return Some(obj_data.clone());
                }
            }
        }
        data
    }
}

/// Bevy asset wrapping a parsed Tiled `Map` and the resolved tile-set textures
/// needed to render tile layers.
#[derive(TypePath, Asset)]
pub struct TiledMap {
    /// Parsed Tiled map data.
    pub map: tiled::Map,
    /// Resolved textures for tile sets, keyed by tileset index.
    pub tilemap_textures: HashMap<usize, TilemapTexture>
}

/// `tiled::ResourceReader` implementation that serves bytes from memory,
/// allowing Tiled to read referenced resources from an in-memory buffer.
struct BytesResourceReader {
    /// Shared byte buffer backing all reads.
    bytes: Arc<[u8]>,
}

impl BytesResourceReader {
    /// Constructs a reader backed by the provided byte slice.
    ///
    /// # Parameters
    /// * `bytes` - Source data to expose via the reader.
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: Arc::from(bytes),
        }
    }
}

impl tiled::ResourceReader for BytesResourceReader {
    /// In-memory cursor over the shared bytes.
    type Resource = Cursor<Arc<[u8]>>;
    /// I/O error type propagated by the reader.
    type Error = std::io::Error;

    /// Returns a new cursor for the requested path, ignoring the path and
    /// always serving the same in-memory bytes.
    ///
    /// # Parameters
    /// * `_path` - Ignored; Tiled requests a resource path.
    fn read_from(&mut self, _path: &Path) -> Result<Self::Resource, Self::Error> {
        Ok(Cursor::new(self.bytes.clone()))
    }
}

/// Errors that can occur while loading Tiled assets.
#[derive(Debug, Error)]
pub enum TiledAssetLoaderError {
    /// Wrapper around underlying I/O failures from resource reading/parsing.
    #[error("Tiled asset loading error: {0}")]
    Io(#[from] std::io::Error),
}

/// Bevy asset loader for `.tmx` Tiled maps backed by an in-memory
/// `ResourceReader`. Produces a `TiledMap` asset with resolved tileset
/// textures.
///
/// Loads the TMX, builds the tileset texture map, and returns a `TiledMap`.
pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    type Asset = TiledMap;
    type Settings = ();
    type Error = TiledAssetLoaderError;

    /// Reads TMX bytes, parses the map, and collects tileset textures.
    ///
    /// # Parameters
    /// * `reader` - Async reader providing the TMX file bytes.
    /// * `_settings` - Loader settings (unused).
    /// * `load_context` - Context used to resolve/queue dependent assets.
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

/// Mapping from Tiled layer indices to spawned Bevy entities that back those
/// layers. Used to despawn/rebuild layers when reprocessing a map instance.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    /// Map: layer index -> layer root entity.
    pub storage: HashMap<u32, Entity>,
}

/// Component holding a handle to a `TiledMap` asset instance for an entity
/// that represents a map in the world.
#[derive(Component, Default)]
pub struct TiledMapHandle(pub Handle<TiledMap>);

/// Load-state flag component for a map entity to avoid duplicate processing.
#[derive(Component, Default)]
pub struct TiledMapLoaded(pub bool);

/// Bundle for spawning a Tiled map instance with storage, state, transform,
/// and render settings required by bevy_ecs_tilemap.
#[derive(Bundle, Default)]
pub struct TiledMapBundle {
    /// Handle to the `TiledMap` asset.
    pub tiled_map: TiledMapHandle,
    /// Per-layer entity storage.
    pub storage: TiledLayersStorage,
    /// One-shot loaded flag.
    pub load_state: TiledMapLoaded,
    /// Local transform for the map root.
    pub transform: Transform,
    /// Global transform cache.
    pub global_transform: GlobalTransform,
    /// Tilemap render configuration (culling, z-ordering, etc.).
    pub render_settings: TilemapRenderSettings
}

/// System that instantiates all layers from a loaded `TiledMap`, wiring up
/// image layers, tile layers, and object layers. Also refreshes/despawns any
/// previously spawned layer entities, writes collision data, and triggers
/// registered object-layer loader systems.
///
/// # Parameters
/// * `commands` - Command buffer used to spawn/despawn map and tile entities.
/// * `maps` - Asset storage for resolved `TiledMap` assets.
/// * `tile_storage_query` - Access to existing `TileStorage` per layer.
/// * `map_query` - Target map entity and its handle/load-state/storage/render settings.
/// * `object_layers` - Registry for parsed object layers and their loader systems.
/// * `level_data` - Shared level metadata: map, collision mask, image layers.
/// * `asset_server` - For loading image-layer textures referenced by the map.
#[coverage(off)]
fn process_maps(
    mut commands: Commands,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &mut TileStorage)>,
    mut map_query: Query<(
        &TiledMapHandle,
        &mut TiledMapLoaded,
        &mut TiledLayersStorage,
        &TilemapRenderSettings,
    )>,
    mut object_layers: ResMut<ObjectLayers>,
    mut level_data: ResMut<LevelData>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((map_handle, mut load_state, mut layer_storage, render_settings)) = map_query.single_mut() {
        if load_state.0 {
            return;
        }

        if let Some(tiled_map) = maps.get(&map_handle.0) {
            level_data.map = Some(tiled_map.map.clone());
            level_data.image_layers.clear();
            load_state.0 = true;

            for layer_entity in layer_storage.storage.values() {
                if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                    for tile in layer_tile_storage.iter().flatten() {
                        commands.entity(*tile).despawn();
                    }
                }
                commands.entity(*layer_entity).despawn();
            }
            layer_storage.storage.clear();

            for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                let offset_x = layer.offset_x;
                let offset_y = layer.offset_y;

                match layer.layer_type() {
                    tiled::LayerType::Objects(object_layer) => {
                        let data: Vec<ObjectData> = object_layer.object_data().iter().cloned().collect();
                        object_layers.layer_data.insert(layer.name.clone(), data);

                        if let Some(&system) = object_layers.loader_systems.get(&layer.name) {
                            commands.run_system(system);
                            debug!("Loaded system for layer {}", layer.name);
                        } else {
                            warn!("No System fond for ( {:?} )", layer.name);
                        }
                    }

                    tiled::LayerType::Image(image_layer) => {
                        if let Some(img) = &image_layer.image {
                            if let Some(path) = img.source.to_str() {
                                let texture: Handle<Image> = asset_server.load(path);

                                let opacity = layer.opacity;
                                let mut color = Color::WHITE.with_alpha(opacity);
                                if let Some(tint) = layer.tint_color {
                                    let a = tint.alpha;
                                    let r = tint.red;
                                    let g = tint.green;
                                    let b = tint.blue;
                                    color = Color::srgba(
                                        r as f32 / 255.0,
                                        g as f32 / 255.0,
                                        b as f32 / 255.0,
                                        (a as f32 / 255.0) * opacity,
                                    );
                                }

                                let map_px_w = tiled_map.map.width as f32 * tiled_map.map.tile_width as f32;
                                let map_px_h = tiled_map.map.height as f32 * tiled_map.map.tile_height as f32;

                                let iw = img.width as f32;
                                let ih = img.height as f32;
                                let (iw, ih) = if iw > 0.0 && ih > 0.0 { (iw, ih) } else { (0.0, 0.0) };

                                let parent = commands
                                    .spawn((
                                        Name::new(format!("ImageLayer: {}", layer.name)),
                                        Sprite {
                                            anchor: Anchor::BottomLeft,
                                            image: texture.clone(),
                                            color,
                                            ..Default::default()
                                        },
                                        Transform::from_xyz(offset_x, offset_y, layer_index as f32),
                                        GlobalTransform::IDENTITY,
                                        Visibility::Visible,
                                        InheritedVisibility::VISIBLE,
                                    ))
                                    .id();

                                if (image_layer.repeat_x || image_layer.repeat_y) && iw > 0.0 && ih > 0.0 {
                                    let tiles_x = if image_layer.repeat_x {
                                        ((map_px_w - offset_x).max(0.0) / iw).ceil().max(1.0) as u32
                                    } else {
                                        1
                                    };

                                    let tiles_y = if image_layer.repeat_y {
                                        ((map_px_h - offset_y).max(0.0) / ih).ceil().max(1.0) as u32
                                    } else {
                                        1
                                    };

                                    for iy in 0..tiles_y {
                                        for ix in 0..tiles_x {
                                            if ix == 0 && iy == 0 {
                                                continue;
                                            }
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
                                                Transform::from_xyz(dx, dy, layer_index as f32),
                                                GlobalTransform::IDENTITY,
                                                Visibility::Visible,
                                                InheritedVisibility::VISIBLE,
                                                ChildOf(parent),
                                            ));
                                        }
                                    }
                                }

                                layer_storage.storage.insert(layer_index as u32, parent);
                                debug!("Spawned ImageLayer: {}", layer.name);
                            } else {
                                warn!("Image layer '{}' is not Supported.", layer.name);
                            }
                        } else {
                            warn!("Image layer '{}' has no image yet!.", layer.name);
                        }
                    }

                    _ => {}
                }
            }

            // ---------- (2) PRO TILESET: TILE-LAYER ----------
            for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index) else {
                    continue;
                };

                let tile_size = TilemapTileSize {
                    x: tileset.tile_width as f32,
                    y: tileset.tile_height as f32,
                };

                let tile_spacing = TilemapSpacing {
                    x: tileset.spacing as f32,
                    y: tileset.spacing as f32,
                };

                for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                    let offset_x = layer.offset_x;
                    let offset_y = layer.offset_y;

                    if let tiled::LayerType::Tiles(tile_layer) = layer.layer_type() {
                        if let tiled::TileLayer::Finite(layer_data) = tile_layer {
                            let map_size = TilemapSize {
                                x: tiled_map.map.width,
                                y: tiled_map.map.height,
                            };

                            if layer.name.starts_with("Collision") {
                                level_data.collision_map = vec![0; map_size.x as usize * map_size.y as usize];
                            }

                            let grid_size = TilemapGridSize {
                                x: tiled_map.map.tile_width as f32,
                                y: tiled_map.map.tile_height as f32,
                            };

                            let map_type = match tiled_map.map.orientation {
                                tiled::Orientation::Hexagonal => TilemapType::Hexagon(HexCoordSystem::Row),
                                tiled::Orientation::Isometric => TilemapType::Isometric(IsoCoordSystem::Diamond),
                                tiled::Orientation::Staggered => TilemapType::Isometric(IsoCoordSystem::Staggered),
                                tiled::Orientation::Orthogonal => TilemapType::Square,
                            };

                            let mut tile_storage = TileStorage::empty(map_size);
                            let layer_entity = commands.spawn_empty().id();

                            for x in 0..map_size.x {
                                for y in 0..map_size.y {
                                    // Tiled -> Bevy Y flip
                                    let mapped_y = (tiled_map.map.height - 1 - y) as i32;
                                    let mapped_x = x as i32;

                                    let Some(layer_tile) = layer_data.get_tile(mapped_x, mapped_y) else {
                                        continue;
                                    };

                                    // Nur Tiles dieses Tilesets
                                    if tileset_index != layer_tile.tileset_index() {
                                        continue;
                                    }

                                    let Some(layer_tile_data) = layer_data.get_tile_data(mapped_x, mapped_y) else {
                                        continue;
                                    };

                                    let texture_index = match tilemap_texture {
                                        TilemapTexture::Single(_) => layer_tile.id(),
                                    };

                                    let tile_pos = TilePos { x, y };
                                    let tile_entity = commands
                                        .spawn(TileBundle {
                                            position: tile_pos,
                                            tilemap_id: TilemapId(layer_entity),
                                            texture_index: TileTextureIndex(texture_index),
                                            flip: TileFlip {
                                                x: layer_tile_data.flip_h,
                                                y: layer_tile_data.flip_v,
                                                d: layer_tile_data.flip_d,
                                            },
                                            ..default()
                                        })
                                        .id();

                                    tile_storage.set(&tile_pos, tile_entity);

                                    if layer.name.starts_with("Collision") {
                                        level_data.collision_map
                                            [(mapped_x + mapped_y * map_size.x as i32) as usize] = 1;
                                    }
                                }
                            }

                            commands.entity(layer_entity).insert(TilemapBundle {
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
                            });

                            layer_storage.storage.insert(layer_index as u32, layer_entity);
                        }
                    }
                }
            }
        }
    }
}