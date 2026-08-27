use engine_core::Resource;

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

pub mod components {
    use engine_core::Component;

    #[derive(Component, Clone, Debug, Default)]
    pub struct TempCamera;
}

#[derive(Resource, Debug, Default)]
pub struct GameContext;

pub fn init() -> GameContext {
    GameContext
}
