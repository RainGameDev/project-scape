use engine_core::{component, ecs::entities::Entity, Resource};

#[component]
pub struct TempCamera;
#[component]
pub struct Player;

#[derive(Resource, Debug, Clone, Copy)]
pub struct MenuCamera(pub Option<Entity>);
