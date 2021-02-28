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

uniform vec2 dims;

vec3 sampleHemisphere(vec3 normal, vec2 uv) {
    uv.x = 2.0 * uv.x - 1.0;
    float a = PI * 2.0 * uv.y;
    vec2 b = sqrt(1.0 - uv.x*uv.x) * vec2(cos(a), sin(a));
    return normalize(normal + vec3(b.x, b.y, uv.x));
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    uint random_index = ray_index; uint random_index2 = (ray_index + 1) % uint(dims.x + dims.y * dims.x);

    RawRayHit rhit = ray_hit[ray_index];

    vec3 position = rhit.pos.xyz;
    vec3 normal = rhit.normal_dist.xyz;

    vec2 rng = vec2(random_ssbo[random_index], random_ssbo[random_index2]);
    normal = sampleHemisphere(normal, rng);

    RawRay ray;
    ray.pos = vec4(position, 0.0);
    ray.dir = vec4(normal, 0.0);
    ray_ssbo[ray_index] = ray;
}
