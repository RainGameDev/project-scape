use crate::ui::containers::inventory::InventoryItem;
use crate::ui::containers::item_pickup::ItemPickup;
use crate::{
    components::{MenuCamera, Player},
    ui::containers::Container,
};
use anyhow::Result;
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
    input::InputManager,
    nalgebra::Vector3,
    physics::{
        collider::{Collider, ColliderShape},
        gravity::Gravity,
        velocity::Velocity,
    },
    rendering::core::model::GpuMesh,
    start, update,
    window::window_manager::{MouseMode, WindowManager},
};
use game_data::registry::GameRegistry;

pub use ui::containers::inventory::InventoryUi;

pub mod ui;

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

impl GameState {
    pub fn is_playing(&self) -> bool {
        matches!(self, GameState::Playing)
    }

    pub fn is_main_menu(&self) -> bool {
        matches!(self, GameState::MainMenu)
    }
}

pub mod components;
pub mod player;

#[derive(Resource, Debug, Default)]
pub struct GameContext {
    pub registry: GameRegistry,
}

pub fn init() -> GameContext {
    let registry = load_registry().expect("failed to load game registry");
    GameContext { registry }
}

fn load_registry() -> anyhow::Result<GameRegistry> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let game_data_dir = std::path::Path::new(manifest).join("data");
    let base_data_dir = std::path::Path::new(manifest).join("../game_data/data");

    GameRegistry::load_from_dirs(&[game_data_dir, base_data_dir])
}

#[update]
pub fn toggle_pause(
    mut game_state: ResMut<GameState>,
    mut window_manager: ResMut<WindowManager>,
    input: Res<InputManager>,
) -> anyhow::Result<()> {
    if !input.just_pressed("Pause") {
        return Ok(());
    }
    match *game_state {
        GameState::Playing => {
            *game_state = GameState::Paused;
            window_manager.change_mouse_mode(MouseMode::Noop);
        }
        GameState::Paused => {
            *game_state = GameState::Playing;
            window_manager.change_mouse_mode(MouseMode::LockedInvisible);
        }
        GameState::MainMenu => {}
    }
    Ok(())
}

#[start]
pub fn start(commands: &mut Commands) -> Result<()> {
    commands.add_resource(InventoryUi::default());

    let registry = load_registry()?;
    for (id, def) in registry.items {
        commands.add_asset(def, id);
    }

    Ok(())
}

#[update]
pub fn start_playing(
    game_state: Res<GameState>,
    mut menu_camera: ResMut<MenuCamera>,
    mut window_manager: ResMut<WindowManager>,
    assets: Assets<GpuMesh>,
    game_context: Res<GameContext>,
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

    if let Some(iron_sword) = game_context.registry.item("apostasy:Iron Sword") {
        commands.add_component(
            cube,
            ItemPickup {
                item: InventoryItem::from_def(iron_sword.clone()),
            },
        );
    }

    let player = commands.spawn();
    commands.add_component(
        player,
        Transform::from_position(Vector3::new(0.0, 1.0, 0.0)),
    );
    let mut player_velocity = Velocity::zero();
    player_velocity.inertia_tensor = Vector3::zeros();
    commands.add_component(player, player_velocity);
    commands.add_component(player, Container::default());
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
