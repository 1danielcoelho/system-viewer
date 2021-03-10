use web_sys::WebGlTexture;
use crate::managers::resource::material::ShaderDefine;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextureUnit {
    BaseColor = 0,
    MetallicRoughness = 1,
    Normal = 2,
    Emissive = 3,
    Occlusion = 4,
}
impl TextureUnit {
    pub fn get_define(&self) -> ShaderDefine {
        match *self {
            TextureUnit::BaseColor => ShaderDefine::HasBasecolorTexture,
            TextureUnit::MetallicRoughness => ShaderDefine::HasMetallicroughnessTexture,
            TextureUnit::Normal => ShaderDefine::HasNormalTexture,
            TextureUnit::Emissive => ShaderDefine::HasEmissiveTexture,
            TextureUnit::Occlusion => ShaderDefine::HasOcclusionTexture,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub num_channels: u8,
    pub gl_format: u32,
    pub is_cubemap: bool,
    pub gl_handle: Option<WebGlTexture>,
}

