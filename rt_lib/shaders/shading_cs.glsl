#version 450
#include "common.glsl"

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

layout(std430, binding = 1) buffer rayhit_input {
    RawRayHit ray_hit[];
};

uniform vec2 dims;

#include "brdf/mat.glsl"
#include "brdf/lambert.glsl"

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRayHit rhit = ray_hit[ray_index];

    Material mat;
    mat.albedo = vec3(0.8, 0.9, 0.7);

    vec3 light = normalize(vec3(-1.0, -1.0, 1.0));
    vec3 view = vec3(0.0);
    vec3 normal = rhit.normal_dist.xyz;
    vec3 final = lambert(mat, light, view, normal, vec3(0.0), vec3(0.0));

    //Get the current pixel. This will be (0,0,0) for the first raywave, and the total of all previous raywaves otherwise.
    vec3 current = imageLoad(img_output, ivec2(gl_GlobalInvocationID.xy)).rgb;

    //Store the current + final, because to combine all the raywaves, we simply need to add the result together.
    //Every ray only spawns 1 new ray, so we can always just add them together.
    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(current + final, 1.0));
}
