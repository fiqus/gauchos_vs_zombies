use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub current: usize,
    pub max: usize,
}

#[derive(Component)]
pub struct Gaucho;

#[derive(Component)]
pub struct Zombie;

#[derive(Component, Deref, DerefMut)]
pub struct HitReaction(pub Vec2);

#[derive(Component, Deref, DerefMut)]
pub struct Damage(pub usize);
