use engine_core::component;

#[component]
pub struct Consumable {
    pub uses: u32,
}

#[component]
pub struct Enchantment {
    pub name: String,
    pub magnitude: f32,
}
