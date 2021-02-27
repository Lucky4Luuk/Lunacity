#version 450
layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

struct RawRayHit {
    vec4 pos;
    vec4 normal_dist;
};

layout(std430, binding = 1) buffer rayhit_input {
    RawRayHit ray_hit[];
};

uniform vec2 dims;

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRayHit rhit = ray_hit[ray_index];

    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(1.0,vec2(0.0),1.0));
    // float grad = float(ray_index) / (1280.0*720.0);
    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(grad,vec2(0.0),1.0));
    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(rhit.normal_dist.xyz, 1.0));
}
