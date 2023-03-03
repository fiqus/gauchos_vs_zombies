mod animations;
mod assets;
mod components;
pub mod resources;
mod systems;

use std::ops::Deref;

use animations::{AnimationBundle, AnimationPlugin, GauchoAnimationResource};
use assets::ImageAssets;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_asset_loader::prelude::{LoadingState, LoadingStateAppExt};
use bevy_ecs_tilemap::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use noise::SuperSimplex;

use bevy_rapier2d::prelude::*;
use components::{Gaucho, HitReaction};

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
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Next)
                .with_collection::<ImageAssets>(),
        )
        .add_state(GameState::Loading)
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Next)
                .with_system(systems::gaucho::sprite_movement)
                .with_system(
                    systems::camera::camera_movement.after(systems::gaucho::sprite_movement),
                )
                .with_system(systems::gaucho::attack)
                // .with_system(update_bullet_direction)
                .with_system(systems::zombies::check_collisions)
                .with_system(systems::zombies::spawn_wave)
                .with_system(systems::zombies::update_zombies)
                // .with_system(move_zombies)
                .with_system(systems::chunk::spawn_chunks_around_camera)
                .with_system(systems::chunk::despawn_outofrange_chunks),
        )
        .insert_resource(resources::WaveSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Repeating,
        )))
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: systems::chunk::RENDER_CHUNK_SIZE,
        })
        .insert_resource(resources::BulletTimer(Timer::from_seconds(
            0.01,
            TimerMode::Repeating,
        )))
        .insert_resource(resources::ChunkManager::default())
        .run();
}

fn setup(
    mut commands: Commands,
    gaucho_resource: Res<GauchoAnimationResource>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.25;
    commands.spawn(camera);
    // Use only the subset of sprites in the sheet that make up the run animation
    commands
        .spawn(Into::<AnimationBundle>::into(
            gaucho_resource.deref().to_owned(),
        ))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(4.0, 7.0))
        .insert(TransformBundle::from(Transform::from_translation(
            Vec3::new(0.0, 0.0, 1.0),
        )))
        .insert(HitReaction(Vec2::ZERO))
        .insert(Gaucho);

    let noise_fn = SuperSimplex::new(0);
    commands.insert_resource(resources::Noise(Box::new(noise_fn)));
    let wind = asset_server.load("sounds/wind.ogg");
    audio.play_with_settings(wind, PlaybackSettings::LOOP.with_volume(0.3));
}
