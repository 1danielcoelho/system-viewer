in vec3 a_position;
in vec3 a_normal;
in vec3 a_tangent;
in vec4 a_color;
in vec2 a_uv0;
in vec2 a_uv1;

uniform mat4 u_wv_trans;
uniform mat4 u_wv_inv_transp_trans;
uniform mat4 u_wvp_trans;

out vec3 v_pos_c;
out vec3 v_normal_c;
out vec3 v_tangent_c;
out vec4 v_color;
out vec2 v_uv0;
out vec2 v_uv1;

void main() {
    v_pos_c = (u_wv_trans * vec4(a_position, 1.0)).xyz;
    v_normal_c = normalize((u_wv_inv_transp_trans * vec4(a_normal, 0.0)).xyz);
    v_tangent_c = normalize((u_wv_inv_transp_trans * vec4(a_tangent, 0.0)).xyz);
    v_color = a_color;
    v_uv0 = a_uv0;
    v_uv1 = a_uv1;

    gl_Position = u_wvp_trans * vec4(a_position, 1.0);
}
