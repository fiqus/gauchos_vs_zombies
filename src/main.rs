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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, States)]
enum GameState {
    Loading,
    Next,
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Loading
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Gauchos vs Zombies"),
                        ..Default::default()
                    }),
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
        .add_state::<GameState>()
        .add_loading_state(LoadingState::new(GameState::Loading).continue_to_state(GameState::Next))
        .add_collection_to_loading_state::<_, ImageAssets>(GameState::Loading)
        .add_system(setup.in_schedule(OnEnter(GameState::Loading)))
        .add_systems(
            (
                systems::gaucho::sprite_movement,
                systems::gaucho::attack,
                systems::camera::camera_movement.after(systems::gaucho::sprite_movement),
                systems::zombies::check_collisions,
                systems::zombies::spawn_wave,
                systems::zombies::update_zombies,
                systems::chunk::spawn_chunks_around_camera,
                systems::chunk::despawn_outofrange_chunks,
            )
                .in_set(OnUpdate(GameState::Next)),
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
