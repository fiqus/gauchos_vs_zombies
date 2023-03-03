use bevy::{
    math::{vec2, vec3},
    prelude::*,
};
use std::{f32::consts::PI, ops::Deref};

use crate::{
    animations::{Animation, AnimationBundle, FaconAnimationResource},
    components::{Damage, Gaucho, HitReaction},
};

use bevy_rapier2d::prelude::*;

pub fn attack(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    gaucho: Query<Entity, With<Gaucho>>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    facon_resource: Res<FaconAnimationResource>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let shoot = asset_server.load("sounds/knife_attack.ogg");
        audio.play(shoot);

        let window = windows.get_primary().unwrap();
        if let Some(position) = window.cursor_position() {
            let screen_center = vec2(window.width() / 2., window.height() / 2.);
            let mouse_coordinates = (position - screen_center).normalize() * 10.0;
            let is_looking_up = mouse_coordinates.y > 5.0;
            let is_looking_down = mouse_coordinates.y < -5.0;
            let is_looking_left = mouse_coordinates.x < 0.0;

            let direction = match (is_looking_up, is_looking_down, is_looking_left) {
                (true, _, _) => PI / 2.,
                (_, true, _) => PI * 1.5,
                (_, _, true) => PI,
                (false, false, false) => 0.,
            };

            let mut facon_bundle = Into::<AnimationBundle>::into(facon_resource.deref().to_owned());
            facon_bundle.sprite.transform.translation = vec3(11., 0., 0.);
            facon_bundle
                .sprite
                .transform
                .rotate_around(Vec3::ZERO, Quat::from_rotation_z(direction));
            let facon = commands
                .spawn(facon_bundle)
                .insert(RigidBody::Fixed)
                .insert(Collider::cuboid(4.0, 5.))
                .insert(Sensor)
                .insert(Damage(40))
                .id();
            commands.entity(gaucho.single()).add_child(facon);
        }
    }
}

pub fn sprite_movement(
    windows: Res<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    mut sprite_position: Query<
        (
            &mut Transform,
            &mut Animation,
            &mut HitReaction,
            &mut TextureAtlasSprite,
        ),
        With<Gaucho>,
    >,
    time: Res<Time>,
) {
    let window = windows.get_primary().unwrap();
    for (mut transform, mut animation, mut hit_reaction, mut sprite) in sprite_position.iter_mut() {
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
        if hit_reaction.length() > 0.001 {
            transform.translation.x += hit_reaction.x;
            transform.translation.y += hit_reaction.y;
            hit_reaction.0 *= 0.75;
            sprite.color = if time.elapsed().as_millis() % 100 < 50 {
                Color::RED
            } else {
                Color::WHITE
            };
        } else {
            hit_reaction.0 = Vec2::ZERO;
            sprite.color = Color::WHITE;
        }
    }
}
