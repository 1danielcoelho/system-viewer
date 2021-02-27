precision highp float;

uniform mat4 u_vp_inv_trans;

uniform samplerCube us_base_color;

in vec2 v_position;

out vec4 out_frag_color;

void main() 
{
    vec4 t = u_vp_inv_trans * vec4(v_position, 1.0, 1.0);
    out_frag_color = 2.0 * texture(us_base_color, normalize(t.xyz));    
}