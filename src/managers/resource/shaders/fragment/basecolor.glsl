precision mediump float;

uniform sampler2D us_basecolor;

varying lowp vec3 v_world_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;

void main() 
{
    gl_FragColor = texture2D(us_basecolor, v_uv0); 
}