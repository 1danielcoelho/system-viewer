attribute vec3 a_position;
attribute vec3 a_normal;
attribute vec3 a_tangent;
attribute vec4 a_color;
attribute vec2 a_uv0;
attribute vec2 a_uv1;

uniform mat4 u_world_trans;
uniform mat4 u_world_trans_inv_transp;
uniform mat4 u_view_proj_trans;

varying lowp vec3 v_world_normal;
varying lowp vec3 v_world_tangent;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;
varying lowp vec3 v_world_pos;

void main() {
    v_world_normal = normalize((u_world_trans_inv_transp * vec4(a_normal, 0.0)).xyz);
    v_world_tangent = normalize((u_world_trans_inv_transp * vec4(a_tangent, 0.0)).xyz);
    v_color = a_color;
    v_uv0 = a_uv0;
    v_uv1 = a_uv1;
    v_world_pos = (u_world_trans * vec4(a_position, 1.0)).xyz;

    gl_Position = u_view_proj_trans * vec4(v_world_pos, 1.0);
}
