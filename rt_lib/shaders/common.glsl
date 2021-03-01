#define PI 3.14159265359

//Raw rayhit for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRayHit {
    vec4 pos_id; //w = object id
    vec4 normal_dist;
    vec4 pixel; //xy = pixel coords
};

//Raw ray for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRay {
    vec4 pos;
    vec4 dir;
    vec4 pixel; //xy = pixel coords
};

struct Ray {
    vec3 pos;
    vec3 dir;
    vec2 pixel; //The pixel this ray is affecting
};

struct RayHit {
    vec3 pos;
    int objectID; //0 = skybox
    vec3 normal; //Surface normal
    float dist;
    vec2 pixel; //The pixel this ray is affecting
};

//Stores the information regarding the closest object
struct MapInfo {
    float dist;
    int objectID;
};

//TODO: Optimize
MapInfo mapMin(MapInfo a, MapInfo b) {
    if (a.dist < b.dist) {
        return a;
    } else {
        return b;
    }
}

#include "raytracing/distance_fields.glsl"

MapInfo map(vec3 pos) {
    MapInfo m = MapInfo(sdSphere(pos, 2.0), 1);
    m = mapMin(m, MapInfo(sdInfHorizPlane(pos - vec3(0.0, -2.0, 0.0)), 2));
    m = mapMin(m, MapInfo(sdSphere(pos - vec3(3.0, 2.0, -0.5), 0.5), 3));
    return m;
}
