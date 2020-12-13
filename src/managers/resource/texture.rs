use web_sys::WebGlTexture;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextureUnit {
    Albedo = 0,
    RoughnessMetallic = 1,
    Opacity = 2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ChannelFormat {
    U8,
}

pub struct Texture {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub num_channels: u8,
    pub channel_format: ChannelFormat,
    pub gl_handle: Option<WebGlTexture>,
}
impl Texture {
    pub fn new() -> Self {
        return Self::default();
    }
}
impl Default for Texture {
    fn default() -> Self {
        return Self {
            name: "texture".to_owned(),
            width: 0,
            height: 0,
            num_channels: 0,
            channel_format: ChannelFormat::U8,
            gl_handle: None,
        };
    }
}
