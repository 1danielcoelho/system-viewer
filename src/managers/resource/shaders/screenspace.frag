precision highp float;

uniform sampler2D us_basecolor;

in vec2 v_position;
in vec2 v_uv0;

out vec4 out_frag_color;

void main() 
{
    #ifdef HAS_BASECOLOR_TEXTURE
        out_frag_color = texture(us_basecolor, v_uv0);
    #else
        //out_frag_color = vec4((v_position.x + 1.0) * 0.5, (v_position.y + 1.0) * 0.5, 0, 1); 
        out_frag_color = vec4(v_uv0.xy, 0, 1); 
    #endif
}