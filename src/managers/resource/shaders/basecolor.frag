precision mediump float;

uniform sampler2D us_basecolor;

in vec3 v_world_normal;
in vec4 v_color;
in vec2 v_uv0;
in vec2 v_uv1;

out vec4 out_frag_color;

void main() 
{
    out_frag_color = texture(us_basecolor, v_uv0); 
}