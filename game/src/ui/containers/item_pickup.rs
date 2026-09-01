use engine_core::component;

use crate::ui::containers::inventory::InventoryItem;

#[component]
pub struct ItemPickup {
    pub item: InventoryItem,
}
