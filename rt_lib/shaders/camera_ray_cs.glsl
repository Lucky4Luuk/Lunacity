#version 450
#include "common.glsl"

layout(local_size_x = DISPATCH_SIZE_X, local_size_y = DISPATCH_SIZE_Y, local_size_z = 1) in;

//Output
layout(std430, binding = 0) buffer ray_buffer {
    RawRay ray_ssbo[];
};

uniform vec2 dims;
uniform mat4 invprojview;
uniform vec2 jitter;

Ray rayFromProjview(vec2 uv) {
    vec2 pos = uv * 2.0 - 1.0;
    float near = 0.02;
    float far = 1024.0;
    vec3 origin = (invprojview * vec4(pos, -1.0, 1.0) * near).xyz;
    vec3 dir = (invprojview * vec4(pos * (far - near), far + near, far - near)).xyz;
    Ray ray;
    ray.pos = origin;
    ray.dir = normalize(dir);
    return ray;
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);

    vec2 pixel_coords = vec2(gl_GlobalInvocationID.xy);
    vec2 uv = pixel_coords / dims + jitter * 0.001;

    Ray ray = rayFromProjview(uv);
    RawRay rray;
    rray.pos = vec4(ray.pos, 0.0);
    rray.dir = vec4(ray.dir, 0.0);
    rray.pixel = vec4(pixel_coords, 0.0, 0.0);
    rray.power = vec4(vec3(1.0), 0.0); //power starts at 1

    ray_ssbo[ray_index] = rray;
}
