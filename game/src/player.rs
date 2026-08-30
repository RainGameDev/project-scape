use anyhow::Result;
use engine_core::{
    Resource,
    ecs::{
        components::engine_components::{camera::GameCamera, transform::Transform},
        query::query::Query,
        systems::{
            DeltaTime,
            param::{Res, ResMut},
        },
    },
    input::InputManager,
    nalgebra::{UnitQuaternion, Vector3},
    physics::velocity::Velocity,
    update,
};

use crate::GameState;
use crate::components::Player;

const MAX_PITCH: f32 = 1.0;
const JUMP_SPEED: f32 = 5.0;

/// Look/move state for the player.
#[derive(Resource, Debug)]
pub struct LookController {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for LookController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.005,
            speed: 10.0,
        }
    }
}

/// Mouse look and WASD movement for the player.
#[update]
pub fn player_controller(
    game_state: Res<GameState>,
    input: Res<InputManager>,
    mut look: ResMut<LookController>,
    delta_time: Res<DeltaTime>,
    players: Query<(&mut Transform, &mut Velocity, &Player)>,
    cameras: Query<(&mut Transform, &GameCamera)>,
) -> Result<()> {
    if !game_state.is_playing() {
        return Ok(());
    }
    let (Some((player_transform, player_velocity, _)), Some((camera_transform, _))) =
        (players.iter().next(), cameras.iter().next())
    else {
        return Ok(());
    };
    let delta = delta_time.0;

    let (mouse_x, mouse_y) = input.mouse_delta();
    look.yaw += mouse_x * look.sensitivity;
    look.pitch = (look.pitch - mouse_y * look.sensitivity).clamp(-MAX_PITCH, MAX_PITCH);

    player_transform.rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), -look.yaw);
    camera_transform.rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), look.pitch);

    let facing = player_transform.forward();
    let forward = Vector3::new(facing.x, 0.0, facing.z);
    let right = forward.cross(&Vector3::y());

    let forward_axis = input.axis("MoveForward") - input.axis("MoveBackward");
    let strafe_axis = input.axis("MoveRight") - input.axis("MoveLeft");

    let move_dir = forward * forward_axis + right * strafe_axis;
    let length_squared = move_dir.norm_squared();
    if length_squared > 1e-6 {
        let move_dir = move_dir / length_squared.sqrt();
        player_transform.position += move_dir * look.speed * delta;
    }

    if input.just_pressed("Jump") && player_velocity.is_grounded {
        player_velocity.linear_velocity.y = JUMP_SPEED;
    }

    Ok(())
}
