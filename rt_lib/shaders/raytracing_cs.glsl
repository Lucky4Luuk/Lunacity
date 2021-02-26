#version 450
layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

struct RawRay {
    vec4 pos;
    vec4 dir;
};

layout(std430, binding = 1) buffer ray_buffer {
    RawRay ray_ssbo[];
};

#include "settings.glsl"
#include "raytracing/distance_fields.glsl"

uniform vec2 dims;
uniform mat4 invprojview;

struct Ray {
    vec3 pos;
    vec3 dir;
};

struct RayHit {
    vec3 pos;
    float dist;
};

float map(vec3 pos) {
    float d = sdSphere(pos, 2.0); //Sphere at origin
    // return min(d, sdInfHorizPlane(pos - vec3(0.0, -2.0, 0.0)) );
    return d;
}

RayHit trace(Ray ray) {
    RayHit hit;
    hit.pos = ray.pos;
    hit.dist = 0.0;

    for (int i = 0; i < MAX_STEPS; i++) {
        float d = map(ray.pos + ray.dir * hit.dist);
        if (d < DIST_PRECISION) { //todo: step scaling based on i and multiplier
            hit.pos = ray.pos + ray.dir * hit.dist;
            break;
        }
        hit.dist += d;
    }
    return hit;
}

//For distance fields.
//For polygonal meshes, the normal will come from the mesh data.
vec3 calcNormal(vec3 p) {
    const float h = 0.0001;
    const vec2 k = vec2(1.0, -1.0);
    return normalize(k.xyy*map(p + k.xyy*h) +
                     k.yyx*map(p + k.yyx*h) +
                     k.yxy*map(p + k.yxy*h) +
                     k.xxx*map(p + k.xxx*h) );
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRay rray = ray_ssbo[ray_index];
    Ray ray;
    ray.pos = rray.pos.xyz;
    ray.dir = rray.dir.xyz;
    RayHit hit = trace(ray);

    vec3 normal = calcNormal(hit.pos);

    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(hit.dist / 255.0), 1.0));
    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(normal, 1.0));
}
