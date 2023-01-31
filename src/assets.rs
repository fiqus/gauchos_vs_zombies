use bevy::prelude::*;
use bevy_asset_loader::prelude::AssetCollection;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "StaticTiles.png")]
    pub tiles: Handle<Image>,
    #[asset(path = "gaucho.png")]
    pub gaucho: Handle<Image>,
    #[asset(path = "zombie.png")]
    pub zombie: Handle<Image>,
}
