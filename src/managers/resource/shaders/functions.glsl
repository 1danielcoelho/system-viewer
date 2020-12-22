#include <constants.glsl>

float sq(float t)
{
    return t * t;
}

vec2 sq(vec2 t)
{
    return t * t;
}

vec3 sq(vec3 t)
{
    return t * t;
}

vec4 sq(vec4 t)
{
    return t * t;
}

float clamped_dot(vec3 x, vec3 y)
{
    return clamp(dot(x, y), 0.0, 1.0);
}

vec3 linear_to_sRGB(vec3 color)
{
    return pow(color, vec3(INV_GAMMA));
}

vec3 sRGB_to_linear(vec3 srgbIn)
{
    return vec3(pow(srgbIn.xyz, vec3(GAMMA)));
}

vec4 sRGB_to_linear(vec4 srgbIn)
{
    return vec4(sRGB_to_linear(srgbIn.xyz), srgbIn.w);
}