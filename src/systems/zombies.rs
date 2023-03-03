use std::ops::Deref;

use crate::animations::{Animation, AnimationBundle, ZombieAnimationResource};
use crate::components::{Damage, Gaucho, Health, HitReaction, Zombie};
use crate::resources;
use bevy::math::{vec2, Vec3Swizzles};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::distributions::Uniform;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

pub fn spawn_wave(
    time: Res<Time>,
    mut timer: ResMut<resources::WaveSpawnTimer>,
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
            let bar = commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgba(0.9, 0.0, 0., 0.8),
                        custom_size: Some(Vec2::new(15.0, 2.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3 {
                        x: 0.,
                        y: 9.,
                        z: 0.,
                    }),
                    ..default()
                })
                .id();
            commands
                .spawn(zombie_bundle)
                .insert(RigidBody::Dynamic)
                .insert(GravityScale(0.0))
                .insert(Collider::cuboid(4.0, 8.0))
                .insert(Velocity::linear(Vec2::ZERO))
                .insert(LockedAxes::ROTATION_LOCKED)
                .insert(HitReaction(Vec2::ZERO))
                .insert(Health {
                    current: 100,
                    max: 100,
                })
                .add_child(bar)
                .insert(Zombie);
        }
    }
}

pub fn update_zombies(
    mut zombies: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Animation,
            &mut HitReaction,
            &mut TextureAtlasSprite,
        ),
        With<Zombie>,
    >,
    gaucho: Query<&Transform, (With<Gaucho>, Without<Zombie>)>,
    time: Res<Time>,
) {
    let gaucho_pos = gaucho.single();
    for (mut zombie_vel, mut zombie_pos, mut animation, mut hit_reaction, mut sprite) in
        zombies.iter_mut()
    {
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

pub fn check_collisions(
    mut commands: Commands,
    weapons: Query<(Entity, &Damage)>,
    mut gaucho: Query<(Entity, &mut HitReaction, &Transform), With<Gaucho>>,
    mut zombies: Query<
        (Entity, &mut HitReaction, &mut Health, &Transform, &Children),
        (With<Zombie>, Without<Gaucho>),
    >,
    mut zombie_children: Query<&mut Sprite>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    rapier_context: Res<RapierContext>,
) {
    let (gaucho, mut gaucho_reaction, gaucho_transform) = gaucho.get_single_mut().unwrap();
    if gaucho_reaction.length() == 0. {
        for (zombie, mut zombie_reaction, _, _, _) in zombies.iter_mut() {
            if let Some(contact_pair) = rapier_context.contact_pair(gaucho, zombie) {
                if contact_pair.has_any_active_contacts() {
                    for manifold in contact_pair.manifolds() {
                        gaucho_reaction.x += manifold.local_n2().x * 5.;
                        gaucho_reaction.y += manifold.local_n2().y * 5.;
                        zombie_reaction.x += manifold.local_n1().x * 5.;
                        zombie_reaction.y += manifold.local_n1().y * 5.;
                    }
                }
            }
        }
    }
    let mut random = thread_rng();

    for (weapon, damage) in weapons.iter() {
        for (zombie, mut zombie_reaction, mut health, zombie_transform, children) in
            zombies.iter_mut()
        {
            if zombie_reaction.length() == 0. {
                if rapier_context.intersection_pair(weapon, zombie) == Some(true) {
                    let zombie_sound = asset_server.load("sounds/zombie.ogg");
                    let impact = asset_server.load("sounds/impact.ogg");

                    audio.play(impact);
                    audio.play(zombie_sound);
                    let damage = (damage.0 as f32 * random.sample(Uniform::new(0.5, 2.))) as usize;
                    if health.current <= damage {
                        commands.entity(zombie).despawn_recursive();
                    } else {
                        health.current -= damage;
                        zombie_reaction.0 += (zombie_transform.translation.xy()
                            - gaucho_transform.translation.xy())
                        .normalize()
                            * 5.;
                        for &child in children.iter() {
                            if let Ok(mut health_sprite) = zombie_children.get_mut(child) {
                                health_sprite.custom_size =
                                    Some(vec2(16. * health.current as f32 / health.max as f32, 2.))
                            }
                        }
                    }
                }
            }
        }
    }
}
