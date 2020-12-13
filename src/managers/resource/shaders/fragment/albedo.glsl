precision mediump float;

uniform sampler2D us_albedo;

varying lowp vec3 v_world_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;

void main() 
{
    gl_FragColor = texture2D(us_albedo, v_uv0); 
}