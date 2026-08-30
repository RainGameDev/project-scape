use engine_core::{
    Resource,
    ecs::{
        commands::Commands,
        components::engine_components::{
            camera::{Camera, GameCamera},
            model_renderer::ModelRenderer,
            transform::Transform,
        },
        systems::param::{Assets, Res, ResMut},
    },
    nalgebra::Vector3,
    physics::{
        collider::{Collider, ColliderShape},
        gravity::Gravity,
        velocity::Velocity,
    },
    rendering::core::model::GpuMesh,
    update,
    window::window_manager::{MouseMode, WindowManager},
};

use crate::components::{MenuCamera, Player};

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        matches!(self, GameState::Playing)
    }
}

pub mod components;
pub mod player;

#[derive(Resource, Debug, Default)]
pub struct GameContext;

pub fn init() -> GameContext {
    GameContext
}

#[update]
pub fn start_playing(
    game_state: Res<GameState>,
    mut menu_camera: ResMut<MenuCamera>,
    mut window_manager: ResMut<WindowManager>,
    assets: Assets<GpuMesh>,
    commands: &mut Commands,
) -> anyhow::Result<()> {
    if !game_state.is_playing() {
        return Ok(());
    }
    let Some(menu) = menu_camera.0.take() else {
        return Ok(());
    };

    window_manager.change_mouse_mode(MouseMode::LockedInvisible);

    commands.despawn(menu);

    let floor = commands.spawn();
    commands.add_component(
        floor,
        Transform {
            scale: Vector3::new(10.0, 0.25, 10.0),
            ..Default::default()
        },
    );
    let mut collider = Collider::new(
        ColliderShape::Cuboid {
            size: Vector3::new(1.0, 1.0, 1.0),
        },
        Vector3::new(0.0, 0.0, 0.0),
    );
    collider.is_static = true;
    commands.add_component(floor, collider);

    let mut velocity = Velocity::zero();
    velocity.mass = 0.0;
    velocity.process = false;
    commands.add_component(floor, velocity);

    if let Some(handle) = assets.get_handle("meshes/cube.glb") {
        commands.add_component(floor, ModelRenderer { model: handle });
    }

    let cube = commands.spawn();
    commands.add_component(
        cube,
        Transform {
            position: Vector3::new(0., 1.25, 0.0),
            scale: Vector3::new(1.0, 2.5, 1.0),
            ..Default::default()
        },
    );
    let mut collider = Collider::new(
        ColliderShape::Cuboid {
            size: Vector3::new(1.0, 1.0, 1.0),
        },
        Vector3::new(0.0, 0.0, 0.0),
    );
    collider.is_static = true;
    commands.add_component(cube, collider);

    let mut velocity = Velocity::zero();
    velocity.mass = 0.0;
    velocity.process = false;
    commands.add_component(cube, velocity);

    if let Some(handle) = assets.get_handle("meshes/cube.glb") {
        commands.add_component(cube, ModelRenderer { model: handle });
    }

    let player = commands.spawn();
    commands.add_component(
        player,
        Transform::from_position(Vector3::new(0.0, 1.0, 0.0)),
    );
    let mut player_velocity = Velocity::zero();
    player_velocity.inertia_tensor = Vector3::zeros();
    commands.add_component(player, player_velocity);
    commands.add_component(
        player,
        Gravity {
            force: -9.81,
            weight: 1.0,
        },
    );
    commands.add_component(player, Player);
    commands.add_component(
        player,
        Collider::new(
            ColliderShape::Cuboid {
                size: Vector3::new(1.0, 2.0, 1.0),
            },
            Vector3::new(0.0, 0.0, 0.0),
        ),
    );

    let play_camera = commands.spawn();
    commands.add_component(
        play_camera,
        Transform::from_position(Vector3::new(0.0, 1.0, 0.0)),
    );
    commands.add_component(play_camera, GameCamera);
    commands.add_component(play_camera, Camera::perspective(90.0, 1.0, 0.001, 1000.0));

    commands.set_parent(play_camera, Some(player));

    Ok(())
}
