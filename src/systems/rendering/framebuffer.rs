use crate::managers::resource::texture::Texture;
use crate::utils::gl::GL;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer};

pub struct Framebuffer {
    handle: Option<WebGlFramebuffer>,
    width: u32,
    height: u32,
    color_tex: Rc<RefCell<Texture>>,
    depth_renderbuf: Option<WebGlRenderbuffer>,
}
impl Framebuffer {
    pub fn new(mut width: u32, mut height: u32, gl: &WebGl2RenderingContext) -> Self {
        width = width.max(1);
        height = height.max(1);

        let handle = gl.create_framebuffer();
        gl.bind_framebuffer(GL::FRAMEBUFFER, handle.as_ref());

        // Color texture
        let color_tex = gl.create_texture();
        gl.bind_texture(GL::TEXTURE_2D, color_tex.as_ref());
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR as i32);
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            width as i32,
            height as i32,
            0,
            GL::RGBA as u32,
            GL::UNSIGNED_BYTE,
            None,
        )
        .unwrap();
        gl.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::COLOR_ATTACHMENT0,
            GL::TEXTURE_2D,
            color_tex.as_ref(),
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
            gl_handle: color_tex,
        }));

        // Depth renderbuffer
        let depth_buf = gl.create_renderbuffer();
        gl.bind_renderbuffer(GL::RENDERBUFFER, depth_buf.as_ref());
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
            depth_buf.as_ref(),
        );

        if gl.check_framebuffer_status(GL::FRAMEBUFFER) != GL::FRAMEBUFFER_COMPLETE {
            log::error!(
                "Failed to create main framebuffer with width {}, height {}",
                width,
                height
            );
        }

        log::info!(
            "Created main framebuffer with width {}, height {}",
            width,
            height
        );

        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.bind_renderbuffer(GL::RENDERBUFFER, None);
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);

        return Self {
            handle,
            width,
            height,
            color_tex: tex,
            depth_renderbuf: depth_buf,
        };
    }

    #[allow(dead_code)]
    pub fn cleanup(&self, gl: &WebGl2RenderingContext) {
        // Delete attachments
        gl.delete_texture(self.color_tex.borrow().gl_handle.as_ref());
        gl.delete_renderbuffer(self.depth_renderbuf.as_ref());

        // Delete our framebuffer
        gl.delete_framebuffer(self.handle.as_ref());
    }

    pub fn get_color_tex(&self) -> &Rc<RefCell<Texture>> {
        return &self.color_tex;
    }

    pub fn resize(&mut self, width: u32, height: u32, gl: &WebGl2RenderingContext) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width.max(1);
        self.height = height.max(1);

        // Resize color texture
        gl.bind_texture(GL::TEXTURE_2D, self.color_tex.borrow().gl_handle.as_ref());
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            self.width as i32,
            self.height as i32,
            0,
            GL::RGBA as u32,
            GL::UNSIGNED_BYTE,
            None,
        )
        .unwrap();
        gl.bind_texture(GL::TEXTURE_2D, None);

        // Resize depth renderbuffer
        gl.bind_renderbuffer(GL::RENDERBUFFER, self.depth_renderbuf.as_ref());
        gl.renderbuffer_storage(
            GL::RENDERBUFFER,
            GL::DEPTH32F_STENCIL8,
            self.width as i32,
            self.height as i32,
        );
        gl.bind_renderbuffer(GL::RENDERBUFFER, None);

        log::info!(
            "Resized framebuffer attachments with width {}, height {}",
            self.width,
            self.height
        );
    }

    pub fn bind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(GL::FRAMEBUFFER, self.handle.as_ref());
        gl.viewport(0, 0, self.width as i32, self.height as i32);
    }

    pub fn unbind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(GL::FRAMEBUFFER, None);
    }
}
