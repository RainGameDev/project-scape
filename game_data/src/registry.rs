use anyhow::{Context, Result};
use engine_core::{
    ecs::components::component_registry::{
        find_component_registration_by_name, inventory, ComponentRegistration,
    },
    log_info,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

use crate::components::item::ItemDef;

pub const REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameRegistry {
    /// Bump when defs change so clients know to resync.
    pub version: u32,
    pub items: HashMap<String, ItemDef>,
}

impl GameRegistry {
    pub fn item(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }

    pub fn load_from_dirs(dirs: &[impl AsRef<Path>]) -> Result<Self> {
        let mut registry = GameRegistry {
            version: REGISTRY_VERSION,
            ..Default::default()
        };

        for dir in dirs {
            let dir = dir.as_ref();
            for (name, bytes) in read_json_files(&dir.join("items"))? {
                let def: ItemDef = json5::from_str(&String::from_utf8_lossy(&bytes))
                    .with_context(|| format!("parsing item def '{name}'"))?;
                for component in def.components.keys() {
                    if find_component_registration_by_case(component).is_none() {
                        anyhow::bail!(
                            "item '{}' references unknown component '{}'",
                            def.qualified_id(),
                            component
                        );
                    }
                }
                log_info!("Importing asset: {}", name);
                registry.items.insert(def.qualified_id(), def);
            }
        }

        Ok(registry)
    }
}

fn find_component_registration_by_case(
    name: &str,
) -> Option<&'static ComponentRegistration> {
    find_component_registration_by_name(name).or_else(|| {
        inventory::iter::<ComponentRegistration>()
            .find(|reg| reg.type_name.eq_ignore_ascii_case(name))
    })
}

fn read_json_files(dir: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()).is_some_and(|e| e == "json5") {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
            files.push((name, bytes));
        }
    }
    Ok(files)
}
