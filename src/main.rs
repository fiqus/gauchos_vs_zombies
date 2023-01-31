mod animations;
mod assets;

use std::ops::Deref;

use animations::{
    Animation, AnimationBundle, AnimationPlugin, GauchoAnimationResource, ZombieAnimationResource,
};
use assets::ImageAssets;
use bevy::math::{vec2, Vec3Swizzles};

use bevy::utils::HashSet;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use noise::{NoiseFn, SuperSimplex};

use bevy_rapier2d::prelude::*;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

const TILE_SIZE: TilemapTileSize = TilemapTileSize { x: 16.0, y: 16.0 };
// For this example, don't choose too large a chunk size.
const CHUNK_SIZE: UVec2 = UVec2 { x: 8, y: 8 };
// Render chunk sizes are set to 4 render chunks per user specified chunk.
const RENDER_CHUNK_SIZE: UVec2 = UVec2 {
    x: CHUNK_SIZE.x * 2,
    y: CHUNK_SIZE.y * 2,
};

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
        .add_plugin(AnimationPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Next)
                .with_collection::<ImageAssets>(),
        )
        .add_state(GameState::Loading)
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(sprite_movement)
                .with_system(camera_movement.after(sprite_movement))
                .with_system(shoot)
                // .with_system(update_bullet_direction)
                .with_system(check_collisions)
                .with_system(spawn_wave)
                .with_system(update_zombies)
                // .with_system(move_zombies)
                .with_system(spawn_chunks_around_camera)
                .with_system(despawn_outofrange_chunks),
        )
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

#[derive(Component, Deref, DerefMut)]
struct HitReaction(Vec2);

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

fn setup(mut commands: Commands, gaucho_resource: Res<GauchoAnimationResource>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.25;
    commands.spawn(camera);
    // Use only the subset of sprites in the sheet that make up the run animation
    commands
        .spawn(Into::<AnimationBundle>::into(
            gaucho_resource.deref().to_owned(),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(8.0))
        .insert(TransformBundle::from(Transform::from_translation(
            Vec3::new(0.0, 0.0, 1.0),
        )))
        .insert(HitReaction(Vec2::ZERO))
        .insert(Gaucho);

    let noise_fn = SuperSimplex::new(0);
    commands.insert_resource(Noise(Box::new(noise_fn)));
}

fn camera_movement(
    gaucho_transform: Query<&Transform, With<Gaucho>>,
    mut camera_position: Query<&mut Transform, (With<Camera>, Without<Gaucho>)>,
) {
    let mut tr = camera_position.single_mut();
    tr.translation = gaucho_transform.single().translation;
}

fn sprite_movement(
    windows: Res<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite_position: Query<(&mut Transform, &mut Animation, &mut HitReaction), With<Gaucho>>,
) {
    let window = windows.get_primary().unwrap();
    for (mut transform, mut animation, mut hit_reaction) in sprite_position.iter_mut() {
        if let Some(position) = window.cursor_position() {
            let screen_center = vec2(window.width() / 2., window.height() / 2.);
            let mouse_coordinates = (position - screen_center).normalize() * 10.0;
            let is_looking_up = mouse_coordinates.y > 5.0;
            let is_looking_down = mouse_coordinates.y < -5.0;
            let is_looking_left = mouse_coordinates.x < 0.0;

            let direction = match (is_looking_up, is_looking_down, is_looking_left) {
                (true, _, _) => "Up",
                (_, true, _) => "Down",
                (_, _, true) => "Left",
                (false, false, false) => "Right",
            };
            let mut speed = Vec2::ZERO;

            if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
                speed.y = 1.0;
            }
            if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
                speed.y = -1.0;
            }
            if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
                speed.x = -1.0;
            }
            if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
                speed.x = 1.0;
            }

            if hit_reaction.length() > 0.001 {
                speed += hit_reaction.0;
                hit_reaction.0 *= 0.25;
            } else {
                hit_reaction.0 = Vec2::ZERO;
            }

            if speed == Vec2::ZERO {
                let state = format!("{direction}Idle",);
                animation.set_state(state);
            } else {
                let state = format!("{direction}Walking",);
                animation.set_state(state);

                speed = speed.normalize() * 2.0;
                transform.translation.x += speed.x;
                transform.translation.y += speed.y;
            }
        }
    }
}

#[derive(Resource)]
struct WaveSpawnTimer(Timer);

fn spawn_wave(
    time: Res<Time>,
    mut timer: ResMut<WaveSpawnTimer>,
    mut commands: Commands,
    zombie_resource: Res<ZombieAnimationResource>,
    gaucho_transform: Query<&Transform, (With<Gaucho>, Without<Camera2d>)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let gaucho_translation = gaucho_transform.get_single().unwrap().translation;
        for _ in 0..5 {
            let x = gaucho_translation.x
                + rng.gen_range(100.0..500.0) * ([-1., 1.].choose(&mut rng)).unwrap();
            let y = gaucho_translation.y
                + rng.gen_range(100.0..500.0) * ([-1., 1.].choose(&mut rng)).unwrap();
            let mut zombie_bundle =
                Into::<AnimationBundle>::into(zombie_resource.deref().to_owned());
            zombie_bundle.sprite.transform.translation.x = x;
            zombie_bundle.sprite.transform.translation.y = y;
            commands
                .spawn(zombie_bundle)
                .insert(RigidBody::Dynamic)
                .insert(GravityScale(0.0))
                .insert(Collider::ball(8.0))
                .insert(Velocity::linear(Vec2::ZERO))
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(HitReaction(Vec2::ZERO))
                .insert(Zombie);
        }
    }
}

fn update_zombies(
    mut zombies: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Animation,
            &mut HitReaction,
        ),
        With<Zombie>,
    >,
    gaucho: Query<&Transform, (With<Gaucho>, Without<Zombie>)>,
) {
    let gaucho_pos = gaucho.single();
    for (mut zombie_vel, mut zombie_pos, mut animation, mut hit_reaction) in zombies.iter_mut() {
        let dir = gaucho_pos.translation - zombie_pos.translation;
        zombie_vel.linvel = Vec2::new(dir.x, dir.y).normalize() * 50.;
        if zombie_vel.linvel.y.abs() > zombie_vel.linvel.x.abs() {
            if zombie_vel.linvel.y > 0. {
                animation.set_state("UpWalking".to_string());
            } else {
                animation.set_state("DownWalking".to_string());
            }
        } else if zombie_vel.linvel.x > 0. {
            animation.set_state("RightWalking".to_string());
        } else {
            animation.set_state("LeftWalking".to_string());
        }
        if hit_reaction.length() > 0.001 {
            zombie_pos.translation.x += hit_reaction.0.x;
            zombie_pos.translation.y += hit_reaction.0.y;
            hit_reaction.0 *= 0.25;
        } else {
            hit_reaction.0 = Vec2::ZERO;
        }
    }
}

fn shoot(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    player_transform: Query<&Transform, With<Gaucho>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let shoot = asset_server.load("sounds/shoot.ogg");
        audio.play(shoot);
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
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(2.5))
                .insert(Sensor)
                .insert(GravityScale(0.0))
                .insert(Velocity {
                    linvel: (position - vec2(window.width() / 2., window.height() / 2.))
                        .normalize()
                        * 100.0,
                    angvel: 0.0,
                })
                .insert(Bullet);
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    bullets: Query<Entity, With<Bullet>>,
    mut gaucho: Query<(Entity, &mut HitReaction), With<Gaucho>>,
    mut zombies: Query<(Entity, &mut HitReaction), (With<Zombie>, Without<Gaucho>)>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    rapier_context: Res<RapierContext>,
) {
    let (gaucho, mut gaucho_reaction) = gaucho.get_single_mut().unwrap();
    for (zombie, mut zombie_reaction) in zombies.iter_mut() {
        if let Some(contact_pair) = rapier_context.contact_pair(gaucho, zombie) {
            if contact_pair.has_any_active_contacts() {
                for manifold in contact_pair.manifolds() {
                    gaucho_reaction.x += manifold.local_n2().x * 50.;
                    gaucho_reaction.y += manifold.local_n2().y * 50.;
                    zombie_reaction.x += manifold.local_n1().x * 10.;
                    zombie_reaction.y += manifold.local_n1().y * 10.;
                }
            }
        }
    }
    for bullet in bullets.iter() {
        for (zombie, _) in zombies.iter() {
            if rapier_context.intersection_pair(bullet, zombie) == Some(true) {
                let zombie_sound = asset_server.load("sounds/zombie.ogg");
                let impact = asset_server.load("sounds/impact.ogg");

                audio.play(impact);
                audio.play(zombie_sound);

                commands.entity(zombie).despawn_recursive();
                commands.entity(bullet).despawn_recursive();
            }
        }
    }
}
