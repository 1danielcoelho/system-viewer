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

vec3 calc_point_light(vec3 light_pos, vec3 light_color, float light_intensity)
{
    vec3 frag_to_light = light_pos - v_world_pos;
    float dist_squared = dot(frag_to_light, frag_to_light);
    float intensity = 10.0 * light_intensity / dist_squared;

    intensity *= dot(normalize(frag_to_light), v_world_normal);
    intensity = clamp(intensity, 0.0, 1.0);
    return light_color * intensity;
}

void main() 
{
    vec3 total_color = vec3(0, 0, 0);

    for(int i = 0; i < MAX_LIGHTS; ++i)
    {
        total_color += calc_point_light(
            u_light_pos_or_dir[i], 
            u_light_colors[i], 
            u_light_intensities[i]
        );
    }

    gl_FragColor = vec4(total_color, 1.0);
}
