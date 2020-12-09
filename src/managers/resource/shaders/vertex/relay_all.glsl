attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec4 a_color;
attribute vec2 a_uv0;
attribute vec2 a_uv1;

uniform mat4 u_transform;

varying lowp vec3 v_normal;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;

void main() {
  v_normal = a_normal;
  v_color = a_color;
  v_uv0 = a_uv0;
  v_uv1 = a_uv1;

  gl_Position = u_transform * vec4(a_position, 1.0);
}
