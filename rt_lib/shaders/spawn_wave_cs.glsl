#version 450
#include "common.glsl"

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

//Input 1
layout(std430, binding = 0) buffer rayhit_input {
    RawRayHit ray_hit[];
};

//Input 2
layout(std430, binding = 1) buffer random_input {
    float random_ssbo[];
};

//Output
layout(std430, binding = 2) buffer ray_buffer {
    RawRay ray_ssbo[];
};

void main() {

}
