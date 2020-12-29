use serde::{Deserialize, Serialize};
use web_sys::WebGlTexture;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureUnit {
    BaseColor = 0,
    MetallicRoughness = 1,
    Normal = 2,
    Emissive = 3,
    Occlusion = 4,
}
impl TextureUnit {
    pub fn get_define(&self) -> &'static str {
        match *self {
            TextureUnit::BaseColor => "#define BASECOLOR_TEXTURE",
            TextureUnit::MetallicRoughness => "#define METALLICROUGHNESS_TEXTURE",
            TextureUnit::Normal => "#define NORMAL_TEXTURE",
            TextureUnit::Emissive => "#define EMISSIVE_TEXTURE",
            TextureUnit::Occlusion => "#define OCCLUSION_TEXTURE",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Texture {
    pub name: String,

    #[serde(skip)]
    pub width: u32,

    #[serde(skip)]
    pub height: u32,

    #[serde(skip)]
    pub num_channels: u8,

    #[serde(skip)]
    pub gl_format: u32,

    #[serde(skip)]
    pub gl_handle: Option<WebGlTexture>,
}
