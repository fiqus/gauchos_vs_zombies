use bevy::math::{vec2, vec3, Vec3Swizzles};
use bevy::sprite::collide_aabb::collide;

use bevy::utils::HashSet;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_asset_loader::prelude::{AssetCollection, LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use noise::{NoiseFn, SuperSimplex};
use rand::distributions::Uniform;
use rand::{thread_rng, Rng};

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
// For this example, don't choose too large a chunk size.
const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };
// Render chunk sizes are set to 4 render chunks per user specified chunk.
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "gaucho.png")]
    pub gaucho: Handle<Image>,
    #[asset(path = "zombie.png")]
    pub zombie: Handle<Image>,
    #[asset(path = "StaticTiles.png")]
    pub tiles: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Loading,
    Next,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: String::from("Gauchos vs Zombies"),
                        ..Default::default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .build()
                .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Next)
                .with_collection::<ImageAssets>(),
        )
        .add_state(GameState::Loading)
        .add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(sprite_movement)
                .with_system(animate_sprite)
                .with_system(shoot)
                .with_system(update_bullet_direction)
                .with_system(check_collisions)
                .with_system(spawn_wave)
                .with_system(update_zombies)
                .with_system(move_zombies)
                .with_system(spawn_chunks_around_camera)
                .with_system(despawn_outofrange_chunks),
        )
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(setup))
        .insert_resource(WaveSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: RENDER_CHUNK_SIZE,
        })
        .insert_resource(BulletTimer(Timer::from_seconds(0.01, TimerMode::Repeating)))
        .insert_resource(ChunkManager::default())
        .run();
}

#[derive(Component)]
struct Gaucho;

#[derive(Component)]
struct Bullet;

#[derive(Resource)]
struct BulletTimer(Timer);

#[derive(Component)]
struct Zombie;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource, Deref)]
struct ZombieTexture(Handle<TextureAtlas>);

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

#[derive(Resource, Deref)]
struct Noise(Box<dyn NoiseFn<f64, 2> + Send + Sync>);

fn spawn_chunk(
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

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
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

fn despawn_outofrange_chunks(
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

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn setup(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.25;
    commands.spawn(camera);
    let texture_handle = image_assets.gaucho.clone();
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 3, 4, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 0, last: 2 };
    commands
        .spawn((
            SpriteSheetBundle {
                texture_atlas: texture_atlas_handle,
                sprite: TextureAtlasSprite::new(animation_indices.first),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        ))
        .insert(Gaucho);

    let texture_atlas = TextureAtlas::from_grid(
        image_assets.zombie.clone(),
        Vec2::new(16.0, 16.0),
        3,
        4,
        None,
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.insert_resource(ZombieTexture(texture_atlas_handle));
    let noise_fn = SuperSimplex::new(0);
    commands.insert_resource(Noise(Box::new(noise_fn)));
}

fn sprite_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_position: Query<&mut Transform, (With<Camera2d>, Without<Gaucho>)>,
    mut sprite_position: Query<
        (
            &mut TextureAtlasSprite,
            &mut Transform,
            &mut AnimationIndices,
        ),
        With<Gaucho>,
    >,
) {
    for (mut sprite, mut transform, mut indices) in sprite_position.iter_mut() {
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
            transform.translation.y += 2.0;
            indices.first = 9;
            indices.last = 11;
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
            transform.translation.y -= 2.0;
            indices.first = 0;
            indices.last = 2;
        }
        if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
            transform.translation.x -= 2.0;
            indices.first = 3;
            indices.last = 5;
        }
        if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
            transform.translation.x += 2.0;
            indices.first = 6;
            indices.last = 8;
        }
        if sprite.index < indices.first || sprite.index > indices.last {
            sprite.index = indices.first
        }
        for mut camera_transform in camera_position.iter_mut() {
            camera_transform.translation.x = transform.translation.x;
            camera_transform.translation.y = transform.translation.y;
        }
    }
}

#[derive(Resource)]
struct WaveSpawnTimer(Timer);

fn spawn_wave(
    time: Res<Time>,
    mut timer: ResMut<WaveSpawnTimer>,
    mut commands: Commands,
    gaucho_transform: Query<&Transform, (With<Gaucho>, Without<Camera2d>)>,
    texture: Res<ZombieTexture>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let gaucho_translation = gaucho_transform.get_single().unwrap().translation;
        for _ in 0..5 {
            let x = gaucho_translation.x
                + rng.gen_range(500.0..1000.0) * rng.sample(Uniform::new(-1., 1.));
            let y = gaucho_translation.y
                + rng.gen_range(500.0..1000.0) * rng.sample(Uniform::new(-1., 1.));
            let animation_indices = AnimationIndices { first: 0, last: 2 };
            commands
                .spawn((
                    SpriteSheetBundle {
                        texture_atlas: texture.clone_weak(),
                        sprite: TextureAtlasSprite::new(animation_indices.first),
                        transform: Transform::from_xyz(x, y, 1.),
                        ..default()
                    },
                    animation_indices,
                    AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                ))
                .insert(Zombie)
                .insert(Velocity(Vec2::new(0., 0.)));
        }
    }
}

fn update_zombies(
    mut zombies: Query<(&mut Velocity, &Transform), With<Zombie>>,
    gaucho: Query<&Transform, With<Gaucho>>,
) {
    let gaucho_pos = gaucho.single();
    for (mut zombie_vel, zombie_pos) in zombies.iter_mut() {
        let dir = gaucho_pos.translation - zombie_pos.translation;
        zombie_vel.0 = Vec2::from([dir.x, dir.y]).normalize() * 1.1;
    }
}

fn move_zombies(
    mut zombies: Query<
        (
            &Velocity,
            &mut Transform,
            &mut TextureAtlasSprite,
            &mut AnimationIndices,
        ),
        With<Zombie>,
    >,
) {
    for (zombie_vel, mut zombie_trans, mut sprite, mut indices) in zombies.iter_mut() {
        zombie_trans.translation += vec3(zombie_vel.0.x, zombie_vel.0.y, 0.0);
        if zombie_vel.0.y.abs() > zombie_vel.0.x.abs() {
            if zombie_vel.0.y > 0. {
                indices.first = 9;
                indices.last = 11;
            } else {
                indices.first = 0;
                indices.last = 2;
            }
        } else if zombie_vel.0.x > 0. {
            indices.first = 6;
            indices.last = 8;
        } else {
            indices.first = 3;
            indices.last = 5;
        }
        if sprite.index < indices.first || sprite.index > indices.last {
            sprite.index = indices.first
        }
    }
}

fn shoot(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    player_transform: Query<&Transform, With<Gaucho>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let player_translation = player_transform.get_single().unwrap().translation;

        let window = windows.get_primary().unwrap();
        if let Some(position) = window.cursor_position() {
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.25, 0.25, 0.75),
                        custom_size: Some(Vec2::new(5.0, 5.0)),
                        ..default()
                    },
                    transform: Transform {
                        translation: player_translation,
                        ..default()
                    },
                    ..default()
                })
                .insert(Bullet)
                .insert(Velocity(
                    (position - vec2(window.width() / 2., window.height() / 2.)).normalize() * 10.0,
                ));
        }
    }
}

fn update_bullet_direction(
    time: Res<Time>,
    mut timer: ResMut<BulletTimer>,
    mut bullet_position: Query<(&mut Transform, &Velocity), With<Bullet>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for (mut transform, velocity) in bullet_position.iter_mut() {
            transform.translation.x += velocity.0.x;
            transform.translation.y += velocity.0.y;
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    bullet_transforms: Query<(Entity, &Sprite, &Transform), With<Bullet>>,
    player_transform: Query<&Transform, With<Gaucho>>,
    zombie_transforms: Query<(Entity, &Transform), With<Zombie>>,
) {
    let player_transform = player_transform.get_single().unwrap();
    for (_, zombie_transform) in zombie_transforms.iter() {
        if let Some(_collision) = collide(
            player_transform.translation,
            vec2(16.0, 16.0),
            zombie_transform.translation,
            vec2(16.0, 16.0),
        ) {
            //println!("perdiste");
        }
    }
    for (bullet, bullet_sprite, bullet_transform) in bullet_transforms.iter() {
        for (zombie, zombie_transform) in zombie_transforms.iter() {
            if let Some(_collision) = collide(
                bullet_transform.translation,
                bullet_sprite.custom_size.unwrap_or(vec2(0.0, 0.0)),
                zombie_transform.translation,
                vec2(16.0, 0.0),
            ) {
                commands.entity(zombie).despawn_recursive();
                commands.entity(bullet).despawn_recursive();
            }
        }
    }
}
