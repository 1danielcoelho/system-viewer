in vec2 a_position;

out vec3 v_position;

void main() 
{
    gl_Position = vec4(a_position, 1.0, 1.0);
}
