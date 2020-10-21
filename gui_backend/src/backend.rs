use crate::*;

pub use egui::{
    app::{App, WebInfo},
    pos2, Srgba,
};

// ----------------------------------------------------------------------------

pub struct WebBackend {
    ctx: Arc<egui::Context>,
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

    /// Returns a master fullscreen UI, covering the entire screen.
    pub fn begin_frame(&mut self, raw_input: egui::RawInput) -> egui::Ui {
        self.frame_start = Some(now_sec());
        self.ctx.begin_frame(raw_input)
    }

    pub fn end_frame(&mut self) -> Result<(egui::Output, egui::PaintJobs), JsValue> {
        let frame_start = self
            .frame_start
            .take()
            .expect("unmatched calls to begin_frame/end_frame");

        let (output, paint_jobs) = self.ctx.end_frame();

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

// TODO: Just use RawInput?
/// Data gathered between frames.
/// Is translated to `egui::RawInput` at the start of each frame.
#[derive(Default)]
pub struct WebInput {
    pub mouse_pos: Option<egui::Pos2>,
    pub mouse_down: bool, // TODO: which button
    pub is_touch: bool,
    pub scroll_delta: egui::Vec2,
    pub events: Vec<egui::Event>,
}

impl WebInput {
    pub fn new_frame(&mut self, pixels_per_point: f32) -> egui::RawInput {
        // Compensate for potential different scale of Egui compared to native.
        let scale = native_pixels_per_point() / pixels_per_point;
        let scroll_delta = std::mem::take(&mut self.scroll_delta) * scale;
        let mouse_pos = self.mouse_pos.map(|mp| pos2(mp.x * scale, mp.y * scale));
        egui::RawInput {
            mouse_down: self.mouse_down,
            mouse_pos,
            scroll_delta,
            screen_size: screen_size_in_native_points().unwrap() * scale,
            pixels_per_point: Some(pixels_per_point),
            time: now_sec(),
            events: std::mem::take(&mut self.events),
        }
    }
}
