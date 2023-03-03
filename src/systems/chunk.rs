use crate::assets::ImageAssets;
use crate::resources::{ChunkManager, Noise};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::TilePos;
use bevy_ecs_tilemap::prelude::{TilemapId, TilemapTexture, TilemapTileSize};
use bevy_ecs_tilemap::tiles::{TileBundle, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;
use rand::{thread_rng, Rng};

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
// For this example, don't choose too large a chunk size.
const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };
// Render chunk sizes are set to 4 render chunks per user specified chunk.
pub const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

pub fn spawn_chunk(
    commands: &mut Commands,
    image_assets: &Res<ImageAssets>,
    chunk_pos: IVec2,
    noise: &Res<Noise>,
) {
    let mut random = thread_rng();
    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());

    // Spawn the elements of the tilemap.

    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let noise_val = ((noise.get([
                chunk_pos.x as f64 * CHUNK_SIZE.x as f64 * TILE_SIZE.x as f64
                    + x as f64 * TILE_SIZE.x as f64 / 100.,
                chunk_pos.y as f64 * CHUNK_SIZE.y as f64 * TILE_SIZE.y as f64
                    + y as f64 * TILE_SIZE.y as f64 / 100.,
            ]) + 1.)
                / 2.) as f32;
            let tile_pos = TilePos { x, y };
            let text_index = TileTextureIndex(if noise_val < 0.8 {
                random.gen_range(0..6)
            } else {
                100
            });
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    texture_index: text_index,
                    ..Default::default()
                })
                .id();
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x,
        chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y,
        0.0,
    ));
    let texture_handle = image_assets.tiles.clone();
    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size: TILE_SIZE.into(),
        size: CHUNK_SIZE.into(),
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size: TILE_SIZE,
        transform,
        ..Default::default()
    });
}

pub fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

pub fn spawn_chunks_around_camera(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
    noise: Res<Noise>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation().xy());
        for y in (camera_chunk_pos.y - 4)..(camera_chunk_pos.y + 4) {
            for x in (camera_chunk_pos.x - 4)..(camera_chunk_pos.x + 4) {
                if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                    chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                    spawn_chunk(&mut commands, &image_assets, IVec2::new(x, y), &noise);
                }
            }
        }
    }
}

pub fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&GlobalTransform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform), With<TilemapId>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation().xy().distance(chunk_pos);
            if distance > 320.0 {
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
