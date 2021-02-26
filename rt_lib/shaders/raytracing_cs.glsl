#version 450
layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

#include "settings.glsl"
#include "raytracing/distance_fields.glsl"

uniform vec2 dims;
uniform mat4 invprojview;

struct Ray {
    vec3 pos;
    vec3 dir;
};

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

struct RayHit {
    vec3 pos;
    float dist;
};

float map(vec3 pos) {
    return sdSphere(pos, 2.0); //Sphere at origin
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
    vec3 pixel_coords = vec3(gl_GlobalInvocationID.xyz);
    vec2 uv = pixel_coords.xy / dims;

    Ray ray = rayFromProjview(uv);
    RayHit hit = trace(ray);

    vec3 normal = calcNormal(hit.pos);

    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(vec3(hit.dist / 255.0), 1.0));
    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(normal, 1.0));

    // imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(uv, 0.0, 1.0));
}
