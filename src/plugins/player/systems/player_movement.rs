use bevy::{prelude::*, input::mouse::MouseMotion};
use bevy_rapier3d::prelude::*;
use super::{super::components::{Player, PlayerCamera, JumpDuration}, InputState};


pub fn movement_system(
    mut player_query: Query<(&mut Velocity, &Player), Without<PlayerCamera>>,
    camera_query: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let (mut velocity, player) = player_query.single_mut();
    let camera_transform = camera_query.single();

    let mut x_axis = 0.0;
    if keyboard.pressed(KeyCode::KeyD) {
        x_axis = 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        x_axis = -1.0;
    }

    let mut z_axis = 0.0;
    if keyboard.pressed(KeyCode::KeyS) {
        z_axis = 1.0;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        z_axis = -1.0;
    }   

    let movement_input = Quat::from_rotation_y(camera_transform.rotation.to_euler(EulerRot::YXZ).0) * Vec3::new(x_axis, 0.0, z_axis);
    let movement_direction = movement_input.normalize_or_zero();

    velocity.linvel = Vec3::new(movement_direction.x * player.speed * time.delta_seconds(), velocity.linvel.y, movement_direction.z * player.speed * time.delta_seconds());
}


pub fn jump_system(
    time: Res<Time>,
    mut player_query: Query<(&mut JumpDuration, &mut Velocity, &Player)>,
    kbd: Res<ButtonInput<KeyCode>>,
) {
    // assume we have exactly one player that jumps with Spacebar
    let (mut jump, mut velocity, player) = player_query.single_mut();

    if kbd.just_pressed(KeyCode::Space) {
        jump.time.reset();
    }

    if kbd.pressed(KeyCode::Space) && jump.time.elapsed_secs() < 0.15 {
        jump.time.tick(time.delta());
        velocity.linvel.y = player.jump_force;
    }
}


pub fn camera_rotation_system(
    windows_query: Query<&Window>,
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera), Without<Player>>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
) {
    let player_transform = player_query.single();

    let (mut camera_transform, mut player_camera) = camera_query.single_mut();
    if let Ok(window) = windows_query.get_single()
    {
        for ev in state.reader_motion.read(&motion) {
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity
            let window_scale = window.height().min(window.width());
            pitch -= (0.0006 * ev.delta.y * window_scale).to_radians();
            yaw -= (0.0006 * ev.delta.x * window_scale).to_radians();
    
            pitch = pitch.clamp(-1.57, 1.57);
    
            // Order is important to prevent unintended roll
            camera_transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }

    player_camera.focus = player_transform.translation;
    camera_transform.translation = player_camera.focus + Vec3::new(0.0, 1.0, 0.0);
}
