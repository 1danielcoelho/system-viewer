in vec3 a_position;
in vec3 a_normal;
in vec3 a_tangent;
in vec4 a_color;
in vec2 a_uv0;
in vec2 a_uv1;

uniform mat4 u_world_trans;
uniform mat4 u_world_trans_inv_transp;
uniform mat4 u_wvp_trans;

out vec3 v_world_normal;
out vec3 v_world_tangent;
out vec4 v_color;
out vec2 v_uv0;
out vec2 v_uv1;
out vec3 v_world_pos;

void main() {
    v_world_normal = normalize((u_world_trans_inv_transp * vec4(a_normal, 0.0)).xyz);
    v_world_tangent = normalize((u_world_trans_inv_transp * vec4(a_tangent, 0.0)).xyz);
    v_color = a_color;
    v_uv0 = a_uv0;
    v_uv1 = a_uv1;
    v_world_pos = (u_world_trans * vec4(a_position, 1.0)).xyz;

    gl_Position = u_wvp_trans * vec4(a_position, 1.0);
}
