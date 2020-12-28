use web_sys::WebGl2RenderingContext;

pub type GL = WebGl2RenderingContext;

#[macro_export]
macro_rules! glc {
    ($ctx:expr, $any:expr) => {
        #[cfg(debug_assertions)]
        while $ctx.get_error() != 0 {} // Not sure why he did this
        $any;
        #[cfg(debug_assertions)]
        while match $ctx.get_error() {
            0 => false,
            err => {
                log::error!("[OpenGL Error] {}", err);
                true
            }
        } {}
    };
}

pub fn setup_gl_context(gl: &WebGl2RenderingContext) {
    gl.enable(GL::BLEND);
    gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
    gl.enable(GL::CULL_FACE);
    gl.cull_face(GL::BACK);
    gl.clear_color(0.0, 0.0, 0.0, 1.0); //RGBA
    gl.clear_depth(1.);

    // Check out how many texture units we support
    let max_combined_tex_units = gl.get_parameter(GL::MAX_COMBINED_TEXTURE_IMAGE_UNITS);
    let max_vert_tex_units = gl.get_parameter(GL::MAX_VERTEX_TEXTURE_IMAGE_UNITS);
    let max_frag_tex_units = gl.get_parameter(GL::MAX_TEXTURE_IMAGE_UNITS);
    log::info!(
        "Max texture units: Vertex shader: {:?}, Fragment shader: {:?}, Combined: {:?}",
        max_vert_tex_units,
        max_frag_tex_units,
        max_combined_tex_units
    );
}
