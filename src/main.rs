use bevy::core::FixedTimestep;
use bevy::math::vec3;
use bevy::prelude::*;
use rand::Rng;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(sprite_movement)
        .add_system(spawn_wave)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_zombies)
                .with_system(move_zombies),
        )
        .insert_resource(WaveSpawnTimer(Timer::from_seconds(1.0, true)))
        .run();
}

#[derive(Component)]
struct Gaucho;

#[derive(Component)]
struct Zombie;

#[derive(Component)]
struct Velocity(Vec2);

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 0.),
            ..default()
        })
        .insert(Gaucho);
}

fn sprite_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite_position: Query<(&mut Gaucho, &mut Transform)>,
) {
    for (_, mut transform) in sprite_position.iter_mut() {
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
            transform.translation.y += 10.0;
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
            transform.translation.y -= 10.0;
        }
        if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
            transform.translation.x -= 10.0;
        }
        if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
            transform.translation.x += 10.0;
        }
    }
}

struct WaveSpawnTimer(Timer);

fn spawn_wave(time: Res<Time>, mut timer: ResMut<WaveSpawnTimer>, mut commands: Commands) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        for _ in 0..5 {
            let x = rng.gen_range(-200.0..200.0);
            let y = rng.gen_range(-200.0..200.0);
            commands
                .spawn()
                .insert(Zombie)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.15, 0.35, 0.15),
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, 0.),
                    ..default()
                })
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

fn move_zombies(mut zombies: Query<(&Velocity, &mut Transform), With<Zombie>>) {
    for (zombie_vel, mut zombie_trans) in zombies.iter_mut() {
        zombie_trans.translation += vec3(zombie_vel.0.x, zombie_vel.0.y, 0.0);
    }
}
