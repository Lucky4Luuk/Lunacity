#version 450
#include "common.glsl"

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

layout(std430, binding = 1) buffer rayhit_input {
    RawRayHit ray_hit[];
};

uniform vec2 dims;

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRayHit rhit = ray_hit[ray_index];

    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(rhit.normal_dist.xyz, 1.0));
}
