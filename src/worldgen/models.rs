use bevy::{asset::{AssetLoader, LoadContext, io::Reader}, platform::collections::HashMap, prelude::*};
use serde::Deserialize;
use thiserror::Error;

use crate::CollectingProgress;

#[derive(Debug, Error)]
pub enum BlockModelsLoaderError {
    #[error("Failed to read block models: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse block models JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default, TypePath)]
pub struct BlockModelsLoader {}

impl AssetLoader for BlockModelsLoader {
    type Asset = BlockModel;

    type Settings = ();

    type Error = BlockModelsLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let models = serde_json::from_slice(&bytes)?;

        Ok(models)
    }
    
    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

#[derive(Resource, Debug, Default)]
pub struct BlockModelsManager {
    models: HashMap<String, BlockModel>,
}

impl BlockModelsManager {
    pub fn get(&self, name: impl Into<String>) -> Option<&BlockModel> {
        self.models.get(&name.into())
    }
}

#[derive(Resource, Default)]
pub struct PendingBlockModels {
    pub handles: Vec<(String, Handle<BlockModel>)>
}

impl BlockModelsManager {}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Faces {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Back = 4,
    Front = 5,
}

impl Faces {
    pub const ALL: [Faces; 6] = [
        Faces::Top,
        Faces::Bottom,
        Faces::Right,
        Faces::Left,
        Faces::Back,
        Faces::Front,
    ];
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvRotation {
    None,
    Rotate90,
    Rotate180,
    Rotate270,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CullingMode {
    Default,
    SameBlock,
    None,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct FaceTexture {
    pub tile: Tile,
    pub rotation: UvRotation,
    pub culling: CullingOptions
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct CullingOptions {
    pub mode: CullingMode
}

#[derive(Clone, Copy, Debug, Deserialize)]
struct FaceOptions {
    top: FaceTexture,
    bottom: FaceTexture,
    right: FaceTexture,
    left: FaceTexture,
    front: FaceTexture,
    back: FaceTexture,
}

#[derive(Asset, Clone, Copy, TypePath, Deserialize, Debug)]
pub struct BlockModel {
    face: FaceOptions,
}

impl BlockModel {
    pub fn get_uvs(&self, face: Faces, atlas_size: UVec2) -> [[f32; 2]; 4] {
        let face_texture = self.get_face_textures(face);
        let tex_x = face_texture.tile.x * 16;
        let tex_y = face_texture.tile.y * 16;

        let tex_width = 16u32;
        let tex_height = 16u32;

        let uv_min = Vec2::new(
            tex_x as f32 / atlas_size.x as f32,
            tex_y as f32 / atlas_size.y as f32,
        );

        let uv_max = Vec2::new(
            (tex_x + tex_width) as f32 / atlas_size.x as f32,
            (tex_y + tex_height) as f32 / atlas_size.y as f32,
        );

        let base_uvs = [
            [uv_min.x, uv_max.y],
            [uv_max.x, uv_max.y],
            [uv_max.x, uv_min.y],
            [uv_min.x, uv_min.y]
        ];

        return rotate_uvs(base_uvs, face_texture.rotation);
    }

    pub fn get_face_textures(&self, face: Faces) -> &FaceTexture {
        return match face {
            Faces::Top => &self.face.top,
            Faces::Bottom => &self.face.bottom,
            Faces::Right => &self.face.right,
            Faces::Left => &self.face.left,
            Faces::Back => &self.face.back,
            Faces::Front => &self.face.front,
        };
    }

    pub fn get_opposite_face_textures(&self, face: Faces) -> &FaceTexture {
        return match face {
            Faces::Top => &self.face.bottom,
            Faces::Bottom => &self.face.top,
            Faces::Right => &self.face.left,
            Faces::Left => &self.face.right,
            Faces::Back => &self.face.front,
            Faces::Front => &self.face.back,
        };
    }
}

fn rotate_uvs(
    uvs: [[f32; 2]; 4],
    rotation: UvRotation,
) -> [[f32; 2]; 4] {
    match rotation {
        UvRotation::None => uvs,
        UvRotation::Rotate90 => [uvs[3], uvs[0], uvs[1], uvs[2]],
        UvRotation::Rotate180 => [uvs[2], uvs[3], uvs[0], uvs[1]],
        UvRotation::Rotate270 => [uvs[1], uvs[2], uvs[3], uvs[0]],
    }
}

pub fn collect_loaded_models(
    mut pending: ResMut<PendingBlockModels>,
    mut models_manager: ResMut<BlockModelsManager>,
    mut collecting_progress: ResMut<CollectingProgress>,
    assets: Res<Assets<BlockModel>>,
) {
    pending.handles.retain(|(name, handle)| {
        let Some(model) = assets.get(handle.id()) else {
            return true;
        };

        models_manager.models.insert(name.clone(), model.clone());
        false
    });

    collecting_progress.block_models_loaded = true;
}