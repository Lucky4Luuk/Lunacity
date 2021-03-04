#version 450
#include "common.glsl"

layout(local_size_x = DISPATCH_SIZE_X, local_size_y = DISPATCH_SIZE_Y, local_size_z = 1) in;

//Output
layout(std430, binding = 0) buffer rayhit_output {
    RawRayHit ray_hit[];
};

//Input
layout(std430, binding = 1) buffer ray_buffer {
    RawRay ray_ssbo[];
};

#include "settings.glsl"
#include "raytracing/distance_fields.glsl"

uniform vec2 dims;

//For distance fields.
//For polygonal meshes, the normal will come from the mesh data.
vec3 calcNormal(vec3 p) {
    const float h = 0.0001;
    const vec2 k = vec2(1.0, -1.0);
    return normalize(k.xyy*map(p + k.xyy*h).dist +
                     k.yyx*map(p + k.yyx*h).dist +
                     k.yxy*map(p + k.yxy*h).dist +
                     k.xxx*map(p + k.xxx*h).dist );
}

RayHit trace(Ray ray) {
    RayHit hit;
    hit.pos = ray.pos;
    hit.objectID = 0;
    hit.normal = vec3(0.0);
    hit.dist = 0.0;
    //Passthrough
    hit.pixel = ray.pixel;
    hit.power = ray.power;

    for (int i = 0; i < MAX_STEPS; i++) {
        MapInfo m = map(ray.pos + ray.dir * hit.dist);
        float d = m.dist;
        if (d < DIST_PRECISION) { //TODO: Step scaling based on i and multiplier
            hit.pos = ray.pos + ray.dir * hit.dist;
            hit.normal = calcNormal(hit.pos); //TODO: Only for distance fields, see comment on calcNormal function
            hit.objectID = m.objectID; //TODO: Actual object ID
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
    ray.pixel = rray.pixel.xy;
    ray.power = rray.dir.w;

    RayHit hit = trace(ray);
    RawRayHit rhit;
    rhit.pos_id = vec4(hit.pos, float(hit.objectID));
    rhit.normal_dist = vec4(hit.normal, hit.dist);
    rhit.pixel = vec4(hit.pixel, 0.0, 0.0);
    rhit.dir_pow = vec4(ray.dir, hit.power);
    rhit.col_mask = rray.col_mask;

    ray_hit[ray_index] = rhit;
}
