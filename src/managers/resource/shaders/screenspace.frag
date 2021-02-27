precision highp float;

in vec2 v_position;

out vec4 out_frag_color;

void main() 
{
    out_frag_color = vec4((v_position.x + 1.0) * 0.5, (v_position.y + 1.0) * 0.5, 0, 1); 
}