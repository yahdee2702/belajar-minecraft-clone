use bevy::{platform::collections::HashMap, prelude::*};

use crate::GameState;

pub const AIR_BLOCK: u8 = 0;

#[derive(Resource, Debug, Default)]
pub struct RegistryManager {
    registered_blocks: HashMap<u8, RegisteredBlock>,
    next_id: u8,
}


#[derive(Debug)]
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

    pub fn register_with_id(
        &mut self, 
        id: u8, 
        name: impl Into<String>,
        model: impl Into<String>,
    ) {
        self.registered_blocks.insert(id, RegisteredBlock {
            name: name.into(),
            model: model.into(),
        });

        self.next_id = id + 1;
    }

    pub fn register(
        &mut self, 
        name: impl Into<String>,
        model: impl Into<String>,
    ) {
        self.register_with_id(self.next_id, name, model);
    }
}

pub fn register_blocks(
    mut registry: ResMut<RegistryManager>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    registry.register_with_id(AIR_BLOCK, "air", "models/air.json");
    registry.register("grass", "models/grass.json");
    registry.register("dirt`", "models/dirt.json");
    registry.register("stone`", "models/stone.json");

    next_state.set(GameState::LoadingAsset);
}