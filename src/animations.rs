use std::collections::HashMap;

use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollectionApp;

use crate::assets::ImageAssets;

#[derive(Bundle)]
pub struct AnimationBundle {
    #[bundle]
    pub sprite: SpriteSheetBundle,
    animation: Animation,
}

#[derive(Component)]
pub struct Animation {
    frame: usize,
    state_animations: HashMap<String, Vec<usize>>,
    state: String,
    repeat: bool,
}

impl Animation {
    pub fn set_state(&mut self, state: String) {
        self.state = state;
    }
}

#[derive(Resource, Deref, DerefMut)]
struct AnimationTimer(Timer);

pub struct AnimationPlugin;
fn animate(
    time: Res<Time>,
    mut commands: Commands,
    mut timer: ResMut<AnimationTimer>,
    mut query: Query<(Entity, &mut Animation, &mut TextureAtlasSprite)>,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        for (entity, mut animation, mut sprite) in query.iter_mut() {
            let curr_animation = animation.state_animations[&animation.state].clone();
            animation.frame += 1;
            if animation.frame >= curr_animation.len() {
                if animation.repeat {
                    animation.frame = 0;
                } else {
                    commands.entity(entity).despawn_recursive();
                    continue;
                }
            }
            sprite.index = curr_animation[animation.frame];
        }
    }
}

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(animate)
            .insert_resource(AnimationTimer(Timer::from_seconds(
                0.1,
                TimerMode::Repeating,
            )))
            .init_collection::<ImageAssets>()
            .init_resource::<GauchoAnimationResource>()
            .init_resource::<FaconAnimationResource>()
            .init_resource::<ZombieAnimationResource>();
    }
}

#[derive(Resource, Clone)]
pub struct GauchoAnimationResource {
    texture: Handle<TextureAtlas>,
    state_animations: HashMap<String, Vec<usize>>,
}

impl FromWorld for GauchoAnimationResource {
    fn from_world(world: &mut World) -> Self {
        let image_asset = &world.get_resource::<ImageAssets>().unwrap().gaucho;
        let texture_handle = image_asset.clone();
        let mut texture_atlasses = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 3, 4, None, None);
        let texture_atlas_handle = texture_atlasses.add(texture_atlas);
        let mut state_animations = HashMap::new();
        state_animations.insert("DownIdle".to_string(), vec![1]);
        state_animations.insert("DownWalking".to_string(), vec![0, 1, 2, 1]);
        state_animations.insert("LeftIdle".to_string(), vec![4]);
        state_animations.insert("LeftWalking".to_string(), vec![3, 4, 5, 4]);
        state_animations.insert("RightIdle".to_string(), vec![7]);
        state_animations.insert("RightWalking".to_string(), vec![6, 7, 8, 7]);
        state_animations.insert("UpIdle".to_string(), vec![10]);
        state_animations.insert("UpWalking".to_string(), vec![9, 10, 11, 10]);
        GauchoAnimationResource {
            texture: texture_atlas_handle,
            state_animations,
        }
    }
}

impl From<GauchoAnimationResource> for AnimationBundle {
    fn from(resource: GauchoAnimationResource) -> Self {
        Self {
            sprite: SpriteSheetBundle {
                texture_atlas: resource.texture,
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
            animation: Animation {
                frame: 0,
                state_animations: resource.state_animations,
                state: "DownWalking".to_string(),
                repeat: true,
            },
        }
    }
}

#[derive(Resource, Clone)]
pub struct ZombieAnimationResource {
    texture: Handle<TextureAtlas>,
    state_animations: HashMap<String, Vec<usize>>,
}

impl FromWorld for ZombieAnimationResource {
    fn from_world(world: &mut World) -> Self {
        let image_asset = &world.get_resource::<ImageAssets>().unwrap().zombie;
        let texture_handle = image_asset.clone();
        let mut texture_atlasses = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 3, 4, None, None);
        let texture_atlas_handle = texture_atlasses.add(texture_atlas);
        let mut state_animations = HashMap::new();
        state_animations.insert("DownIdle".to_string(), vec![1]);
        state_animations.insert("DownWalking".to_string(), vec![0, 1, 2, 1]);
        state_animations.insert("LeftIdle".to_string(), vec![4]);
        state_animations.insert("LeftWalking".to_string(), vec![3, 4, 5, 4]);
        state_animations.insert("RightIdle".to_string(), vec![7]);
        state_animations.insert("RightWalking".to_string(), vec![6, 7, 8, 7]);
        state_animations.insert("UpIdle".to_string(), vec![10]);
        state_animations.insert("UpWalking".to_string(), vec![9, 10, 11, 10]);
        ZombieAnimationResource {
            texture: texture_atlas_handle,
            state_animations,
        }
    }
}

impl From<ZombieAnimationResource> for AnimationBundle {
    fn from(resource: ZombieAnimationResource) -> Self {
        Self {
            sprite: SpriteSheetBundle {
                texture_atlas: resource.texture,
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
            animation: Animation {
                frame: 0,
                state_animations: resource.state_animations,
                state: "DownWalking".to_string(),
                repeat: true,
            },
        }
    }
}

#[derive(Resource, Clone)]
pub struct FaconAnimationResource {
    texture: Handle<TextureAtlas>,
    state_animations: HashMap<String, Vec<usize>>,
}

impl FromWorld for FaconAnimationResource {
    fn from_world(world: &mut World) -> Self {
        let image_asset = &world.get_resource::<ImageAssets>().unwrap().facon;
        let texture_handle = image_asset.clone();
        let mut texture_atlasses = world.get_resource_mut::<Assets<TextureAtlas>>().unwrap();
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 1, 3, None, None);
        let texture_atlas_handle = texture_atlasses.add(texture_atlas);
        let mut state_animations = HashMap::new();
        state_animations.insert("".to_string(), vec![0, 1, 2]);
        FaconAnimationResource {
            texture: texture_atlas_handle,
            state_animations,
        }
    }
}

impl From<FaconAnimationResource> for AnimationBundle {
    fn from(resource: FaconAnimationResource) -> Self {
        Self {
            sprite: SpriteSheetBundle {
                texture_atlas: resource.texture,
                sprite: TextureAtlasSprite::new(0),
                transform: Transform::from_xyz(0., 0., 1.),
                ..default()
            },
            animation: Animation {
                frame: 0,
                state_animations: resource.state_animations,
                state: "".to_string(),
                repeat: false,
            },
        }
    }
}
