use bevy::prelude::*;

use crate::Gaucho;

pub fn camera_movement(
    gaucho_transform: Query<&Transform, With<Gaucho>>,
    mut camera_position: Query<&mut Transform, (With<Camera>, Without<Gaucho>)>,
) {
    let mut tr = camera_position.single_mut();
    tr.translation = gaucho_transform.single().translation;
}
