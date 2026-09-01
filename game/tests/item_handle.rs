#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::AtomicU32;

    use engine_core::ecs::World;
    use engine_core::ecs::commands::Commands;
    use game_data::{Handle, ItemDef};

    fn def(name: &str) -> ItemDef {
        ItemDef {
            name: name.into(),
            namespace: "apostasy".into(),
            categories: vec!["weapon".into()],
            icon: None,
            components: Default::default(),
        }
    }

    #[test]
    fn item_def_handle_registers_and_resolves() {
        let mut commands = Commands::new(Arc::new(AtomicU32::new(0)));
        let item = def("Iron Sword");
        commands.add_asset(item.clone(), "apostasy:Iron Sword".to_string());
        commands.add_asset(def("Steel Shield"), "apostasy:Steel Shield".to_string());

        let mut world = World::new();
        commands.apply(&mut world);

        let handle: Handle<ItemDef> = world
            .get_asset_handle::<ItemDef>("apostasy:Iron Sword")
            .expect("handle should be registered");
        let resolved = world.get_asset(handle).expect("asset should resolve");
        assert_eq!(resolved.name, "Iron Sword");
    }
}
