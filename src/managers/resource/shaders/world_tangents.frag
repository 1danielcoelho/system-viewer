precision highp float;

in vec3 v_world_normal;
in vec3 v_world_tangent;
in vec4 v_color;
in vec2 v_uv0;
in vec2 v_uv1;

out vec4 out_frag_color;

void main() 
{
    out_frag_color = abs(vec4(normalize(v_world_tangent), 1.0)); 
}