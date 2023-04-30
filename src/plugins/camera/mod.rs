use bevy::prelude::*;

use self::systems::{setup_light_system, setup_camera_system};

pub mod components;
mod systems;
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_startup_system(setup_camera_system)
            .add_startup_system(setup_light_system)
            .add_startup_system(setup_camera_system.in_base_set(CoreSet::PostUpdate));
    }
}