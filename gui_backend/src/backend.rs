use crate::*;

pub use egui::{
    app::{App, WebInfo},
    pos2, Srgba,
};

// ----------------------------------------------------------------------------

pub struct WebBackend {
    pub ctx: Arc<egui::Context>,
    painter: webgl::Painter,
    previous_frame_time: Option<f32>,
    frame_start: Option<f64>,
    last_save_time: Option<f64>,
}

impl WebBackend {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let ctx = egui::Context::new();
        load_memory(&ctx);
        Ok(Self {
            ctx,
            painter: webgl::Painter::new(canvas_id)?,
            previous_frame_time: None,
            frame_start: None,
            last_save_time: None,
        })
    }

    /// id of the canvas html element containing the rendering
    pub fn canvas_id(&self) -> &str {
        self.painter.canvas_id()
    }

    pub fn begin_frame(&mut self, raw_input: egui::RawInput) {
        self.frame_start = Some(now_sec());
        self.ctx.begin_frame(raw_input)
    }

    pub fn end_frame(&mut self) -> Result<(egui::Output, egui::PaintJobs), JsValue> {
        let frame_start = self
            .frame_start
            .take()
            .expect("unmatched calls to begin_frame/end_frame");

        let (output, paint_commands) = self.ctx.end_frame();
        let paint_jobs = self.ctx.tesselate(paint_commands);

        self.auto_save();

        let now = now_sec();
        self.previous_frame_time = Some((now - frame_start) as f32);

        Ok((output, paint_jobs))
    }

    pub fn paint(&mut self, paint_jobs: egui::PaintJobs) -> Result<(), JsValue> {
        let bg_color = egui::color::TRANSPARENT; // Use background css color.
        self.painter.paint_jobs(
            bg_color,
            paint_jobs,
            &self.ctx.texture(),
            self.ctx.pixels_per_point(),
        )
    }

    pub fn auto_save(&mut self) {
        let now = now_sec();
        let time_since_last_save = now - self.last_save_time.unwrap_or(std::f64::NEG_INFINITY);
        const AUTO_SAVE_INTERVAL: f64 = 5.0;
        if time_since_last_save > AUTO_SAVE_INTERVAL {
            self.last_save_time = Some(now);
            save_memory(&self.ctx);
        }
    }

    pub fn painter_debug_info(&self) -> String {
        self.painter.debug_info()
    }
}

impl egui::app::TextureAllocator for webgl::Painter {
    fn alloc(&mut self) -> egui::TextureId {
        self.alloc_user_texture()
    }

    fn set_srgba_premultiplied(
        &mut self,
        id: egui::TextureId,
        size: (usize, usize),
        srgba_pixels: &[Srgba],
    ) {
        self.set_user_texture(id, size, srgba_pixels);
    }

    fn free(&mut self, id: egui::TextureId) {
        self.free_user_texture(id)
    }
}

// ----------------------------------------------------------------------------

/// Data gathered between frames.
#[derive(Default)]
pub struct WebInput {
    /// Is this a touch screen? If so, we ignore mouse events.
    pub is_touch: bool,

    pub raw: egui::RawInput,
}

impl WebInput {
    pub fn new_frame(&mut self) -> egui::RawInput {
        egui::RawInput {
            screen_size: screen_size_in_native_points().unwrap(),
            pixels_per_point: Some(native_pixels_per_point()),
            time: now_sec(),
            ..self.raw.take()
        }
    }
}

// ----------------------------------------------------------------------------

use std::sync::atomic::Ordering::SeqCst;

pub struct NeedRepaint(std::sync::atomic::AtomicBool);

impl Default for NeedRepaint {
    fn default() -> Self {
        Self(true.into())
    }
}

impl NeedRepaint {
    pub fn fetch_and_clear(&self) -> bool {
        self.0.swap(false, SeqCst)
    }

    pub fn set_true(&self) {
        self.0.store(true, SeqCst);
    }
}

impl egui::app::RepaintSignal for NeedRepaint {
    fn request_repaint(&self) {
        self.0.store(true, SeqCst);
    }
}
