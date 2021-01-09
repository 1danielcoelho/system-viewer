precision highp float;

in vec3 v_pos_c;
in vec3 v_normal_c;
in vec3 v_tangent_c;
in vec4 v_color;
in vec2 v_uv0;
in vec2 v_uv1;

out vec4 out_frag_color;

void main() 
{
    out_frag_color = abs(vec4(normalize(v_normal_c), 1.0)); 
}