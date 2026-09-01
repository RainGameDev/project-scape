use engine_core::assets::Asset;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// `ItemDef` can be stored in the engine's asset store, which makes `Handle<ItemDef>`
/// usable for referencing an item (e.g. picking it up or interacting with it).
impl Asset for ItemDef {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub namespace: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub components: HashMap<String, Value>,
}

impl ItemDef {
    pub fn qualified_id(&self) -> String {
        format!("{}:{}", self.namespace, self.name)
    }

    /// Returns the icon sprite name if the icon references a `sprite:` resource,
    /// e.g. `"sprite:mana_crystal"` -> `Some("mana_crystal")`.
    pub fn sprite_icon(&self) -> Option<&str> {
        let icon = self.icon.as_deref()?;
        icon.strip_prefix("sprite:").filter(|r| !r.is_empty())
    }

    /// Case insensitive check for whether a component is present.
    pub fn has_component(&self, name: &str) -> bool {
        self.components
            .keys()
            .any(|key| key.eq_ignore_ascii_case(name))
    }

    /// Case insensitive lookup of a component's raw JSON value.
    pub fn get_component(&self, name: &str) -> Option<&Value> {
        self.components
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
    }
}
