use std::sync::Arc;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::GameState;

pub const AIR_BLOCK: u8 = 0;

#[derive(Resource, Clone, Debug, Default)]
pub struct RegistryManager {
    registered_blocks: HashMap<u8, RegisteredBlock>,
    next_id: u8,
}

#[derive(Clone, Debug, Default)]
pub struct RegisteredBlock {
    pub name: String,
    pub model: String,
}

impl RegistryManager {
    pub fn get(&self, id: u8) -> Option<&RegisteredBlock> {
        self.registered_blocks.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u8, &RegisteredBlock)> {
        self.registered_blocks.iter()
    }

    pub fn register_with_id(&mut self, id: u8, name: impl Into<String>, model: impl Into<String>) {
        self.registered_blocks.insert(
            id,
            RegisteredBlock {
                name: name.into(),
                model: model.into(),
            },
        );

        self.next_id = id + 1;
    }

    pub fn register(&mut self, name: impl Into<String>, model: impl Into<String>) {
        self.register_with_id(self.next_id, name, model);
    }
}

#[derive(Resource, Debug, Default)]
pub struct SharedRegistryManager {
    pub registry_manager: Arc<RegistryManager>
}

impl SharedRegistryManager {
    pub fn get(&self, id: u8) -> Option<&RegisteredBlock> {
        self.registry_manager.get(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u8, &RegisteredBlock)> {
        self.registry_manager.iter()
    }
}

pub fn register_blocks(
    mut commands: Commands,
    mut registry: ResMut<RegistryManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    registry.register_with_id(AIR_BLOCK, "air", "models/air.json");
    registry.register("grass", "models/grass.json");
    registry.register("dirt", "models/dirt.json");
    registry.register("stone", "models/stone.json");
    registry.register("glass", "models/glass.json");
    registry.register("plank", "models/plank.json");
    registry.register("log", "models/log.json");
    registry.register("leave", "models/leave.json");


    commands.insert_resource(SharedRegistryManager {
        registry_manager: Arc::new(registry.clone()),
    });
    commands.remove_resource::<RegistryManager>();

    next_state.set(GameState::LoadingAsset);
}
