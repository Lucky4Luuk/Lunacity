#version 450
layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
// layout(rgba32f, binding = 0) uniform image2D img_output;

//Raw rayhit for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRayHit {
    vec4 pos;
    vec4 normal;
    float dist;
};

layout(std430, binding = 0) buffer rayhit_output {
    RawRayHit ray_hit[];
};

//Raw ray for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
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
    vec3 normal; //Surface normal
    float dist;
};

float map(vec3 pos) {
    float d = sdSphere(pos, 2.0); //Sphere at origin
    // return min(d, sdInfHorizPlane(pos - vec3(0.0, -2.0, 0.0)) );
    return d;
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

RayHit trace(Ray ray) {
    RayHit hit;
    hit.pos = ray.pos;
    hit.dist = 0.0;

    for (int i = 0; i < MAX_STEPS; i++) {
        float d = map(ray.pos + ray.dir * hit.dist);
        if (d < DIST_PRECISION) { //TODO: Step scaling based on i and multiplier
            hit.pos = ray.pos + ray.dir * hit.dist;
            hit.normal = calcNormal(hit.pos); //TODO: Only for distance fields, see comment on calcNormal function
            break;
        }
        hit.dist += d;
    }
    return hit;
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRay rray = ray_ssbo[ray_index];
    Ray ray;
    ray.pos = rray.pos.xyz;
    ray.dir = rray.dir.xyz;

    RayHit hit = trace(ray);
    RawRayHit rhit;
    rhit.pos = vec4(hit.pos, 0.0);
    rhit.normal = vec4(hit.normal, 0.0);
    rhit.dist = hit.dist;

    rhit.normal = vec4(1.0, 0.0, 0.0, 1.0);
    // float grad = float(ray_index) / (1280.0*720.0);
    // rhit.normal = vec4(grad, 1.0-grad, 0.0, 0.0);
    ray_hit[ray_index] = rhit;

    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(hit.dist / 255.0), 1.0));
    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(normal, 1.0));
}
