#version 450
#include "common.glsl"

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(std430, binding = 0) buffer rayhit_output {
    RawRayHit ray_hit[];
};

layout(std430, binding = 1) buffer ray_buffer {
    RawRay ray_ssbo[];
};

#include "settings.glsl"
#include "raytracing/distance_fields.glsl"

uniform vec2 dims;
uniform mat4 invprojview;

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
    hit.normal = vec3(0.0);
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
    rhit.normal_dist = vec4(hit.normal, hit.dist);

    ray_hit[ray_index] = rhit;
}
