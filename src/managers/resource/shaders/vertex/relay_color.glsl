attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec4 a_color;
attribute vec2 a_uv0;
attribute vec2 a_uv1;

uniform mat4 u_transform;

varying lowp vec4 v_color;

void main() {
  v_color = a_color;
  gl_Position = u_transform * vec4(a_position, 1.0);
}
