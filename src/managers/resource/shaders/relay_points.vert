in vec3 a_position;
in vec4 a_color;

out vec4 v_color;

void main() {
    v_color = vec4(a_color.xyz, 1.0);
    gl_Position = vec4(a_position, 1.0);
    gl_PointSize = a_color.w;
}
