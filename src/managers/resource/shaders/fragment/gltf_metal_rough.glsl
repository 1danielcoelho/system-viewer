precision mediump float;

const int MAX_LIGHTS = 8;
const int POINT_LIGHT = 0;
const int DIR_LIGHT = 1;

uniform int u_light_types[MAX_LIGHTS];
uniform vec3 u_light_pos_or_dir[MAX_LIGHTS];
uniform vec3 u_light_colors[MAX_LIGHTS];
uniform float u_light_intensities[MAX_LIGHTS];
uniform sampler2D us_albedo;
uniform sampler2D us_metal_rough;
uniform sampler2D us_normal;
uniform sampler2D us_emissive;
uniform sampler2D us_opacity;
uniform sampler2D us_occlusion;

varying lowp vec3 v_world_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;
varying lowp vec3 v_world_pos;

vec3 calc_point_light(vec3 light_pos, vec3 light_color, float light_intensity)
{
    vec3 frag_to_light = light_pos - v_world_pos;
    float dist_squared = dot(frag_to_light, frag_to_light);
    float intensity = 10.0 * light_intensity / dist_squared;

    intensity *= dot(normalize(frag_to_light), v_world_normal);
    intensity = clamp(intensity, 0.0, 1.0);
    return light_color * intensity;
}

vec3 calc_dir_light(vec3 light_dir, vec3 light_color, float light_intensity)
{
    float intensity = dot(-normalize(light_dir), v_world_normal);
    intensity = clamp(intensity, 0.0, 1.0);
    return light_color * intensity;
}

void main() 
{
    gl_FragColor = texture2D(us_albedo, v_uv0); 
}