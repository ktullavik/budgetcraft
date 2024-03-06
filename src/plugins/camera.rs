use bevy::prelude::*;

use crate::plugins::player::components::PlayerCamera;


pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_startup_system(setup_camera_system)
            .add_systems(Startup, (setup_light_system, setup_camera_system));
    }
}


pub fn setup_camera_system(mut commands: Commands) {

    commands.spawn((Name::new("Camera3D"), Camera3dBundle {
        transform: Transform {
            translation: Vec3::ZERO,
            ..default()
        },
        projection: Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::PI / 2.0,
            ..default()
        }),
        ..default()
    }))
    .insert(PlayerCamera::default());
}


pub fn setup_light_system(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::hex("8fd3ff").unwrap()));
}

