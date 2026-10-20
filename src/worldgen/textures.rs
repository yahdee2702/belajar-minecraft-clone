

use bevy::{image::ImageSampler, prelude::*};

#[derive(Resource, Default)]
pub struct BlockAtlas {
    pub handle: Handle<Image>,
    pub size: UVec2,
}

pub fn prepare_block_atlas(
    mut block_atlas: ResMut<BlockAtlas>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut atlas_image = images.get_mut(&block_atlas.handle).unwrap();
    atlas_image.sampler = ImageSampler::nearest();

    block_atlas.size = atlas_image.size();
}