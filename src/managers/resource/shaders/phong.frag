precision mediump float;

const int MAX_LIGHTS = 8;

const int POINT_LIGHT = 0;
const int DIR_LIGHT = 1;

uniform int u_light_types[MAX_LIGHTS];
uniform vec3 u_light_pos_or_dir[MAX_LIGHTS];
uniform vec3 u_light_colors[MAX_LIGHTS];
uniform float u_light_intensities[MAX_LIGHTS];

in vec3 v_world_normal;
in vec4 v_color;
in vec2 v_uv0;
in vec2 v_uv1;
in vec3 v_world_pos;

out vec4 out_frag_color;

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
    vec3 total_color = vec3(0, 0, 0);

    for(int i = 0; i < MAX_LIGHTS; ++i)
    {
        if (u_light_types[i] == POINT_LIGHT)
        {
            total_color += calc_point_light(
                u_light_pos_or_dir[i], 
                u_light_colors[i], 
                u_light_intensities[i]
            );
        }
        else
        {
            total_color += calc_dir_light(
                u_light_pos_or_dir[i], 
                u_light_colors[i], 
                u_light_intensities[i]
            );
        }
    }

    out_frag_color = vec4(total_color, 1.0);
}
