in vec3 a_position;
in vec3 a_normal;
in vec4 a_color;
in vec2 a_uv0;
in vec2 a_uv1;

uniform mat4 u_world_trans;
uniform mat4 u_view_proj_trans;

out vec4 v_color;

void main() {
  v_color = a_color;
  gl_Position = u_view_proj_trans * u_world_trans * vec4(a_position, 1.0);
}
