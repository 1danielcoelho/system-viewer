precision mediump float;

const int MAX_LIGHTS = 8;

uniform vec3 u_light_pos_or_dir[MAX_LIGHTS];
uniform vec3 u_light_colors[MAX_LIGHTS];
uniform float u_light_intensities[MAX_LIGHTS];

varying lowp vec3 v_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;

void main() {
  gl_FragColor = vec4(u_light_colors[0], 1.0) +
                 vec4(0.00001 * u_light_pos_or_dir[0], 0.0) +
                 vec4(0.00001 * u_light_intensities[0]);
}
