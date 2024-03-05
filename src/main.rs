use bevy::{prelude::*, window::{PresentMode, WindowResolution}};
use bevy_rapier3d::prelude::{RapierPhysicsPlugin, NoUserData};
use plugins::{camera::CameraPlugin, menu::MenuPlugin, player::PlayerPlugin, world::WorldPlugin};


mod plugins;

pub const CHUNK_WIDTH: usize = 8;
pub const CHUNK_HEIGHT: usize = 256;
pub const CHUNK_VOL: usize = CHUNK_WIDTH * CHUNK_WIDTH * CHUNK_HEIGHT;
pub const RENDER_DISTANCE: i32 = 24;


#[derive(Default, Resource, Debug, Eq, PartialEq, States, Hash, Clone)]
enum GameState {
    Running,
    #[default]
    Stopped
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()).set(WindowPlugin {
            primary_window: Some(Window {
                title: "BudgetCraft".to_string(),
                present_mode: PresentMode::AutoVsync,
                resolution: WindowResolution::new(1280.0, 720.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())

        // Init state before our own plugins.
        .init_state::<GameState>()

        .add_plugins(CameraPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(MenuPlugin)
        .run();
}
