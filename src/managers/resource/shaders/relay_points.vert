in vec4 a_position;
in vec4 a_color;

uniform float u_f_coef;

out vec4 v_color;

void main() {
    v_color = vec4(a_color.xyz, 1.0);
    gl_PointSize = a_color.w;

    gl_Position = a_position;

    // Logarithmic depth buffer    
    gl_Position.z = log2(max(1e-6, 1.0 + gl_Position.w)) * u_f_coef - 1.0; 
}
