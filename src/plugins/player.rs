use bevy::prelude::*;

use crate::GameState;

use self::systems::player_movement::{movement_system, jump_system, camera_rotation_system, player_setup_system, lock_cursor, unlock_cursor};
use self::systems::block_manipulation::{block_breaking_system, block_placing_system};

pub(crate) mod systems;
pub(crate) mod components;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(OnEnter(GameState::Running), player_setup_system)
            .add_systems(OnExit(GameState::Running), unlock_cursor)
            .add_systems(Update, (
                lock_cursor,
                camera_rotation_system,
                movement_system,
                jump_system,
                block_breaking_system,
                block_placing_system
            ).run_if(in_state(GameState::Running)));
    }
}