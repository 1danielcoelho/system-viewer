precision highp float;

in vec2 v_uv0;

out vec4 out_frag_color;

void main()
{
    out_frag_color = vec4(v_uv0, 0.0, 1.0); 
}