use crate::*;

pub use egui::{pos2, Color32};

// ----------------------------------------------------------------------------

pub struct WebBackend {
    pub ctx: egui::CtxRef,
    painter: Box<dyn Painter>,
    previous_frame_time: Option<f32>,
    frame_start: Option<f64>,
}

impl WebBackend {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let ctx = egui::CtxRef::default();

        let painter: Box<dyn Painter> =
            if let Ok(webgl2_painter) = webgl2::WebGl2Painter::new(canvas_id) {
                console_log("Using WebGL2 backend");
                Box::new(webgl2_painter)
            } else {
                console_log("Falling back to WebGL1 backend");
                Box::new(webgl1::WebGlPainter::new(canvas_id)?)
            };

        Ok(Self {
            ctx,
            painter,
            previous_frame_time: None,
            frame_start: None,
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

    pub fn end_frame(&mut self) -> Result<(egui::Output, Vec<egui::ClippedMesh>), JsValue> {
        let frame_start = self
            .frame_start
            .take()
            .expect("unmatched calls to begin_frame/end_frame");

        let (output, shapes) = self.ctx.end_frame();
        let clipped_meshes = self.ctx.tessellate(shapes);

        let now = now_sec();
        self.previous_frame_time = Some((now - frame_start) as f32);

        Ok((output, clipped_meshes))
    }

    pub fn paint(
        &mut self,
        clear_color: egui::Rgba,
        clipped_meshes: Vec<egui::ClippedMesh>,
    ) -> Result<(), JsValue> {
        self.painter.upload_egui_texture(&self.ctx.texture());
        self.painter.clear(clear_color);
        self.painter
            .paint_meshes(clipped_meshes, self.ctx.pixels_per_point())
    }

    pub fn painter_debug_info(&self) -> String {
        self.painter.debug_info()
    }
}

// ----------------------------------------------------------------------------

/// Data gathered between frames.
#[derive(Default)]
pub struct WebInput {
    /// Is this a touch screen? If so, we ignore mouse events.
    pub is_touch: bool,

    /// Required because we don't get a position on touched
    pub latest_touch_pos: Option<egui::Pos2>,

    pub raw: egui::RawInput,
}

impl WebInput {
    pub fn new_frame(&mut self, canvas_size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Default::default(), canvas_size)),
            pixels_per_point: Some(native_pixels_per_point()), // We ALWAYS use the native pixels-per-point
            time: Some(now_sec()),
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

impl epi::RepaintSignal for NeedRepaint {
    fn request_repaint(&self) {
        self.0.store(true, SeqCst);
    }
}
