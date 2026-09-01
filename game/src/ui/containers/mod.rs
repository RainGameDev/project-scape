use engine_core::component;

use crate::ui::containers::inventory::InventoryItem;

pub mod inventory;
pub mod item_pickup;

#[component]
#[derive(Default)]
pub struct Container {
    pub items: Vec<InventoryItem>,
}

impl Container {
    pub fn add_item(&mut self, item: &InventoryItem) {
        // If the item cant stack add it to the list
        if !item.def.has_component("canstack") {
            self.add_new_item(item);
            return;
        }

        // if it stacks
        for iitem in &mut self.items {
            // add its amount if it exists
            if iitem.def.name == item.def.name && iitem.def.namespace == item.def.namespace {
                iitem.count += item.count;
                return;
            }
        }

        // add it to the list if it doesnt
        self.add_new_item(item);
    }

    pub fn add_new_item(&mut self, item: &InventoryItem) {
        self.items.push(item.clone());
    }
}
