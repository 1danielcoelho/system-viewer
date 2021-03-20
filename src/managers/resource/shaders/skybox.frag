precision highp float;

#include <constants.glsl>
#include <functions.glsl>

uniform float u_exposure_factor;
uniform mat4 u_vp_inv_trans;

uniform samplerCube us_base_color;

in vec2 v_position;

out vec4 out_frag_color;

void main() 
{
    vec4 t = u_vp_inv_trans * vec4(v_position, 1.0, 1.0);

    // Hopefully linear color
    vec3 color = texture(us_base_color, normalize(t.xyz)).xyz;
    color = sRGB_to_linear(color);

    // Exposure
    color *= u_exposure_factor;

    // HACK because the current skybox is way too faint. There is no info on light units
    // so I'll have to calibrate some factor at some point
    color *= 10000.0;

    // Reinhard tonemapping
    color = color / (color + vec3(1.0));    

    // Convert to sRGB
    color = linear_to_sRGB(color);
    out_frag_color = vec4(color, 1.0);    
}