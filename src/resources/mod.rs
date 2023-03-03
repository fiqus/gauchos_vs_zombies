use bevy::prelude::*;
use bevy::utils::HashSet;
use noise::NoiseFn;

#[derive(Resource, Deref)]
pub struct Noise(pub Box<dyn NoiseFn<f64, 2> + Send + Sync>);

#[derive(Default, Debug, Resource)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}

#[derive(Resource)]
pub struct BulletTimer(pub Timer);

#[derive(Resource)]
pub struct WaveSpawnTimer(pub Timer);
