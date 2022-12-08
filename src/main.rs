use bevy::math::{vec2, vec3};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::time::FixedTimestep;
use rand::Rng;

const TIME_STEP: f32 = 1.0 / 60.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        //.add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        //.add_system(sprite_movement)
        //.add_system(camera_movement)
        //.add_system(spawn_wave)
        /*.add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_zombies)
                .with_system(move_zombies),
        )*/
        //.insert_resource(WaveSpawnTimer(Timer::from_seconds(1.0, true)))
        //.add_system(shoot)
        //.insert_resource(BulletTimer(Timer::from_seconds(0.01, true)))
        //.add_system(update_bullet_direction)
        //.add_system(check_collisions)
        .add_system(animate_sprite)
        .run();
}

#[derive(Component)]
struct Gaucho;

#[derive(Component)]
struct Bullet;

struct BulletTimer(Timer);

#[derive(Component)]
struct Zombie;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("gaucho_delante.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(29.0, 29.0), 2, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(2.5)),
            ..default()
        },
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
    ));
}

fn sprite_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite_position: Query<(&mut Gaucho, &mut Transform)>,
) {
    for (_, mut transform) in sprite_position.iter_mut() {
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
            transform.translation.y += 5.0;
        }
        if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
            transform.translation.y -= 5.0;
        }
        if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
            transform.translation.x -= 5.0;
        }
        if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
            transform.translation.x += 5.0;
        }
    }
}

fn camera_movement(
    mut camera_position: Query<&mut Transform, With<Camera2d>>,
    sprite_position: Query<&Transform, (With<Gaucho>, Without<Camera2d>)>,
) {
    let gaucho_transform = sprite_position.get_single().unwrap();
    for mut transform in camera_position.iter_mut() {
        transform.translation.x = gaucho_transform.translation.x;
        transform.translation.y = gaucho_transform.translation.y;
    }
}

struct WaveSpawnTimer(Timer);

/*fn spawn_wave(
    time: Res<Time>,
    mut timer: ResMut<WaveSpawnTimer>,
    mut commands: Commands,
    gaucho_transform: Query<&Transform, (With<Gaucho>, Without<Camera2d>)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let gaucho_translation = gaucho_transform.get_single().unwrap().translation;
        for _ in 0..5 {
            let x = gaucho_translation.x + rng.gen_range(-200.0..200.0);
            let y = gaucho_translation.y + rng.gen_range(-200.0..200.0);
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
}*/

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
                .spawn_bundle(SpriteBundle {
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

/*fn update_bullet_direction(
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
}*/

fn check_collisions(
    mut commands: Commands,
    bullet_transforms: Query<(Entity, &Sprite, &Transform), With<Bullet>>,
    player_transform: Query<(&Sprite, &Transform), With<Gaucho>>,
    zombie_transforms: Query<(Entity, &Sprite, &Transform), With<Zombie>>,
) {
    let (player_sprite, player_transform) = player_transform.get_single().unwrap();
    for (_, zombie_sprite, zombie_transform) in zombie_transforms.iter() {
        if let Some(_collision) = collide(
            player_transform.translation,
            player_sprite.custom_size.unwrap_or(vec2(0.0, 0.0)),
            zombie_transform.translation,
            zombie_sprite.custom_size.unwrap_or(vec2(0.0, 0.0)),
        ) {
            println!("perdiste");
        }
    }
    for (bullet, bullet_sprite, bullet_transform) in bullet_transforms.iter() {
        for (zombie, zombie_sprite, zombie_transform) in zombie_transforms.iter() {
            if let Some(_collision) = collide(
                bullet_transform.translation,
                bullet_sprite.custom_size.unwrap_or(vec2(0.0, 0.0)),
                zombie_transform.translation,
                zombie_sprite.custom_size.unwrap_or(vec2(0.0, 0.0)),
            ) {
                commands.entity(zombie).despawn_recursive();
                commands.entity(bullet).despawn_recursive();
            }
        }
    }
}
