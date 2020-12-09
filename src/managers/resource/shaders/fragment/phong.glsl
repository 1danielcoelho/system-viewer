precision mediump float;

const int MAX_LIGHTS = 8;

uniform vec3 u_light_pos_or_dir[MAX_LIGHTS];
uniform vec3 u_light_colors[MAX_LIGHTS];
uniform float u_light_intensities[MAX_LIGHTS];

varying lowp vec3 v_world_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;
varying lowp vec3 v_world_pos;

void main() {
    vec3 frag_to_light = u_light_pos_or_dir[0] - v_world_pos;
    float dist_squared = dot(frag_to_light, frag_to_light);
    float intensity = 10.0 * u_light_intensities[0] / dist_squared;

    intensity *= dot(normalize(frag_to_light), v_world_normal);
    intensity = clamp(intensity, 0.0, 1.0);

    gl_FragColor = vec4(intensity, intensity, intensity, 1.0);
}
