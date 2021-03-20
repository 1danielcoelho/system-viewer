in vec2 a_position;
in vec2 a_uv0;

out vec2 v_position;
out vec2 v_uv0;

void main() 
{
    v_position = a_position;
    v_uv0 = a_uv0;

    gl_Position = vec4(a_position, 1.0, 1.0);
}
