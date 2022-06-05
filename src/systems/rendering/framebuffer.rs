use crate::managers::resource::texture::Texture;
use crate::utils::gl::GL;
use crate::utils::log::*;
use glow::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Framebuffer {
    handle: Option<glow::Framebuffer>,
    width: u32,
    height: u32,
    color_tex: Rc<RefCell<Texture>>,
    depth_renderbuf: Option<glow::Renderbuffer>,
}
impl Framebuffer {
    pub fn new(mut width: u32, mut height: u32, gl: &glow::Context) -> Self {
        width = width.max(1);
        height = height.max(1);

        unsafe {
            let handle = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(handle));

            // Color texture
            let color_tex = gl.create_texture().unwrap();
            gl.bind_texture(GL::TEXTURE_2D, Some(color_tex));
            gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
            gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
            gl.tex_image_2d(
                GL::TEXTURE_2D,
                0,
                GL::RGBA as i32,
                width as i32,
                height as i32,
                0,
                GL::RGBA as u32,
                GL::UNSIGNED_BYTE,
                None,
            );
            gl.framebuffer_texture_2d(
                GL::FRAMEBUFFER,
                GL::COLOR_ATTACHMENT0,
                GL::TEXTURE_2D,
                Some(color_tex),
                0,
            );

            // Give our material instance sole ownership of the color texture
            let tex = Rc::new(RefCell::new(Texture {
                name: "color_tex".to_string(),
                width,
                height,
                num_channels: 4,
                gl_format: GL::UNSIGNED_BYTE,
                is_cubemap: false,
                gl_handle: Some(color_tex),
            }));

            // Depth renderbuffer
            let depth_buf = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(GL::RENDERBUFFER, Some(depth_buf));
            gl.renderbuffer_storage(
                GL::RENDERBUFFER,
                GL::DEPTH32F_STENCIL8,
                width as i32,
                height as i32,
            );
            gl.framebuffer_renderbuffer(
                GL::FRAMEBUFFER,
                GL::DEPTH_ATTACHMENT,
                GL::RENDERBUFFER,
                Some(depth_buf),
            );

            if gl.check_framebuffer_status(GL::FRAMEBUFFER) != GL::FRAMEBUFFER_COMPLETE {
                error!(
                    LogCat::Resources,
                    "Failed to create main framebuffer with width {}, height {}", width, height
                );
            }

            debug!(
                LogCat::Resources,
                "Created main framebuffer with width {}, height {}", width, height
            );

            gl.bind_texture(GL::TEXTURE_2D, None);
            gl.bind_renderbuffer(GL::RENDERBUFFER, None);
            gl.bind_framebuffer(GL::FRAMEBUFFER, None);

            return Self {
                handle: Some(handle),
                width,
                height,
                color_tex: tex,
                depth_renderbuf: Some(depth_buf),
            };
        }
    }

    // TODO: Obviously use this when dropping the framebuffer
    #[allow(dead_code)]
    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            // Delete attachments
            if let Some(tex_handle) = self.color_tex.borrow().gl_handle {
                gl.delete_texture(tex_handle);
            }
            if let Some(renderbuf) = self.depth_renderbuf {
                gl.delete_renderbuffer(renderbuf);
            }

            // Delete our framebuffer
            if let Some(handle) = self.handle {
                gl.delete_framebuffer(handle);
            }
        }
    }

    pub fn get_color_tex(&self) -> &Rc<RefCell<Texture>> {
        return &self.color_tex;
    }

    pub fn resize(&mut self, width: u32, height: u32, gl: &glow::Context) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width.max(1);
        self.height = height.max(1);

        unsafe {
            // Resize color texture
            gl.bind_texture(GL::TEXTURE_2D, self.color_tex.borrow().gl_handle);
            gl.tex_image_2d(
                GL::TEXTURE_2D,
                0,
                GL::RGBA as i32,
                self.width as i32,
                self.height as i32,
                0,
                GL::RGBA as u32,
                GL::UNSIGNED_BYTE,
                None,
            );
            gl.bind_texture(GL::TEXTURE_2D, None);

            // Resize depth renderbuffer
            gl.bind_renderbuffer(GL::RENDERBUFFER, self.depth_renderbuf);
            gl.renderbuffer_storage(
                GL::RENDERBUFFER,
                GL::DEPTH32F_STENCIL8,
                self.width as i32,
                self.height as i32,
            );
            gl.bind_renderbuffer(GL::RENDERBUFFER, None);
        }

        debug!(
            LogCat::Resources,
            "Resized framebuffer attachments with width {}, height {}", self.width, self.height
        );
    }

    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(GL::FRAMEBUFFER, self.handle);
            gl.viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(GL::FRAMEBUFFER, None);
        }
    }
}
