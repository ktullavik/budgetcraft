use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_rapier3d::prelude::*;

use components::{Player, JumpDuration};

use crate::{GameState, GameGarbage, cleanup};

use self::systems::player_movement::{movement_system, jump_system, camera_rotation_system};
use self::systems::block_manipulation::{block_breaking_system, block_placing_system};
use self::systems::InputState;

pub(crate) mod systems;
pub(crate) mod components;


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            .add_systems(OnEnter(GameState::Running), player_setup)
            .add_systems(OnExit(GameState::Running), (cleanup::<GameGarbage>, unlock_cursor))
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


pub fn player_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let player = commands.spawn((Name::new("Player"), PbrBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 96.0, 0.0),
            ..Default::default()
        },
        ..default()
    }, GameGarbage))
    .id();

    commands.entity(player)
        .insert(Player { speed: 400.0, jump_force: 9.0 })
        .insert(JumpDuration { time: Stopwatch::new()})
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED_Z | LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Y)
        .insert(Collider::capsule_y(0.5, 0.4))
        .insert(ColliderMassProperties::Density(1.0))
        .insert(Velocity {
            linvel: Vec3::new(0.0, 0.0, 0.0),
            angvel: Vec3::new(0.0, 0.0, 0.0),
        })
        .insert(GravityScale(6.0))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled());
    commands.insert_resource(InputState::default());


    commands.spawn((Name::new("CursorImage"), ImageBundle {
        image: asset_server.load("cursor.png").into(),
        style: Style {
            width: Val::Px(16.),
            height: Val::Px(16.),
            justify_self: JustifySelf::Center,
            align_self: AlignSelf::Center,
            ..default()
        },
        ..default()
    }, GameGarbage));
}


pub fn lock_cursor(mut windows_query: Query<&mut Window>) {
    if let Ok(mut window) = windows_query.get_single_mut() {
        let cur_pos = Vec2::new(window.width() / 2., window.height() / 2.);
		window.set_cursor_position(Some(cur_pos));
        window.cursor.visible = false;
    }
}


pub fn unlock_cursor(mut windows_query: Query<&mut Window>) {
    if let Ok(mut window) = windows_query.get_single_mut() {
        window.cursor.visible = true;
    }
}
