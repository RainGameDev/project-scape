use engine_core::{Resource, component, ecs::entities::Entity};
pub mod item_components;

#[component]
pub struct TempCamera;
#[component]
pub struct Player;

#[derive(Resource, Debug, Clone, Copy)]
pub struct MenuCamera(pub Option<Entity>);
