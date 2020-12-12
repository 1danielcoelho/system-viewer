precision mediump float;

varying lowp vec3 v_world_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;

void main() { gl_FragColor = abs(vec4(v_world_normal, 1.0)); }