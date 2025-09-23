use std::collections::HashMap;
use std::io::{Cursor, Error, ErrorKind};
use std::path::Path;
use std::sync::Arc;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy_ecs_tilemap::anchor::TilemapAnchor;
use bevy_ecs_tilemap::map::TilemapId;
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::TilemapBundle;
use bevy_ecs_tilemap::tiles::TileTextureIndex;
use thiserror::Error;
use tiled::DefaultResourceCache;

pub struct TiledModule;

impl Plugin for TiledModule {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(TiledLoader);
        app.init_asset::<TiledMap>();
        app.add_systems(Update, process_maps);
    }
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

fn process_maps(
    mut commands: Commands,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &mut TileStorage)>,
    mut map_query: Query<(&TiledMapHandle, &mut TiledMapLoaded, &mut TiledLayersStorage, &TilemapRenderSettings)>
) {
    if let Ok((map_handle, mut load_state, mut layer_storage, render_settings))
        = map_query.single_mut() {
        if load_state.0 {
            return;
        }

        if let Some(tiled_map) = maps.get(&map_handle.0) {
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
                        tiled::LayerType::Tiles(tile_layer) => {
                            if let tiled::TileLayer::Finite(layer_data) = tile_layer {
                                let map_size = TilemapSize {
                                    x: tiled_map.map.width,
                                    y: tiled_map.map.height
                                };

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