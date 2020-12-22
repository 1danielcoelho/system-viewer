// Reference for most of this file:
// https://github.com/KhronosGroup/glTF-Sample-Viewer/blob/master/src/shaders/pbr.frag

precision mediump float;

#include <brdf.glsl>
#include <constants.glsl>
#include <functions.glsl>

uniform int u_light_types[MAX_LIGHTS];
uniform vec3 u_light_pos_or_dir[MAX_LIGHTS];
uniform vec3 u_light_colors[MAX_LIGHTS];
uniform float u_light_intensities[MAX_LIGHTS];

uniform vec3 u_world_camera_pos;

uniform vec4 u_basecolor_factor;
uniform float u_metallic_factor;
uniform float u_roughness_factor;
uniform vec3 u_emissive_factor;

uniform sampler2D us_basecolor;
uniform sampler2D us_metal_rough;
uniform sampler2D us_normal;
uniform sampler2D us_emissive;
uniform sampler2D us_occlusion;

varying lowp vec3 v_world_normal;
varying lowp vec3 v_world_tangent;
varying lowp vec4 v_color;
varying lowp vec2 v_uv0;
varying lowp vec2 v_uv1;
varying lowp vec3 v_world_pos;

vec4 get_base_color()
{
    vec4 base_color = u_basecolor_factor;

    #ifdef BASECOLOR_TEXTURE
        base_color *= sRGB_to_linear(texture2D(us_basecolor, v_uv0));
    #endif

    return base_color;// TODO: Vertexcolor, but only when set: * v_color;
}

vec3 get_normal()
{
    #ifdef NORMAL_TEXTURE
        vec3 normal = texture2D(us_normal, v_uv0).rgb * 2.0 - vec3(1.0);
        // normal *= vec3(u_normal_scale, u_normal_scale, 1.0);
        normal = normalize(normal);

        vec3 bitangent = cross(normal, v_world_tangent);

        return mat3(v_world_tangent, bitangent, normal) * normal;
    #else 
        return v_world_normal;
    #endif
}

void main() 
{
    vec3 v = normalize(u_world_camera_pos - v_world_pos);
    vec3 n = get_normal();

    vec4 base_color = get_base_color();
    float ior = 1.5;
    vec3 f0 = vec3(0.04);
    float perceptual_roughness = u_roughness_factor;
    float metallic = u_metallic_factor;

    #ifdef METALLICROUGHNESS_TEXTURE
        vec4 sample = texture2D(us_metal_rough, v_uv0);
        perceptual_roughness *= sample.g;
        metallic *= sample.b;
    #endif

    vec3 albedo = mix(base_color.rgb *( vec3(1.0) - f0), vec3(0), metallic);
    f0 = mix(f0, base_color.rgb, metallic);

    perceptual_roughness = clamp(perceptual_roughness, 0.0, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);

    float alpha_roughness = perceptual_roughness * perceptual_roughness;
    float reflectance = max(max(f0.r, f0.g), f0.b);
    vec3 f90 = vec3(clamp(reflectance * 50.0, 0.0, 1.0));

    // Lighting
    vec3 diffuse_color = vec3(0);
    vec3 specular_color = vec3(0);
    for(int i = 0; i < MAX_LIGHTS; ++i)
    {
        vec3 pos_to_light = -u_light_pos_or_dir[i];
        float attenuation = 1.0;

        if (u_light_types[i] == POINT_LIGHT) {
            pos_to_light = u_light_pos_or_dir[i] - v_world_pos;
            attenuation = 1.0 / dot(pos_to_light, pos_to_light);
        }
        
        vec3 intensity = attenuation * u_light_intensities[i] * u_light_colors[i];

        vec3 l = normalize(pos_to_light);
        vec3 h = normalize(l + v);
        float n_dot_l = clamped_dot(n, l);
        float n_dot_v = clamped_dot(n, v);
        float n_dot_h = clamped_dot(n, h);
        float l_dot_h = clamped_dot(l, h);
        float v_dot_h = clamped_dot(v, h);

        if (n_dot_l > 0.0 || n_dot_v > 0.0) {
            diffuse_color += intensity * n_dot_l * BRDF_lambertian(f0, f90, albedo, v_dot_h);
            specular_color += intensity * n_dot_l * BRDF_specularGGX(f0, f90, alpha_roughness, v_dot_h, n_dot_l, n_dot_v, n_dot_h);
        }
    }

    vec3 emissive_color = u_emissive_factor;
    #ifdef EMISSIVE_TEXTURE
        emissive_color = sRGB_to_linear(texture2D(us_emissive, v_uv0)).rgb;
    #endif 

    vec3 color = emissive_color + diffuse_color + specular_color;

    #ifdef OCCLUSION_TEXTURE
        color *= texture2D(us_occlusion, v_uv0).r;        
    #endif

    gl_FragColor = vec4(linear_to_sRGB(color), base_color.a); 
}