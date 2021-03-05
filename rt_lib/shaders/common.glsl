#define PI 3.14159265359
#define TAU 2*PI

//Raw rayhit for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRayHit {
    vec4 pos_id; //w = object id
    vec4 normal_dist; //xyz = normal, w = distance
    vec4 pixel; //xy = pixel coords
    vec4 dir; //xyz = ray dir
    vec4 power; //rgb = power
};

//Raw ray for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRay {
    vec4 pos; //xyz = position
    vec4 dir; //xyz = ray dir
    vec4 pixel; //xy = pixel coords
    vec4 power; //rgb = power
};

struct Ray {
    vec3 pos;
    vec3 dir;
    vec2 pixel; //The pixel this ray is affecting
    vec3 power;
};

struct RayHit {
    vec3 pos;
    int objectID; //0 = skybox
    vec3 normal; //Surface normal
    float dist;
    vec2 pixel; //The pixel this ray is affecting
    vec3 power;
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
    // MapInfo m = MapInfo(sdSphere(pos, 2.0), 1);
    // m = mapMin(m, MapInfo(sdInfHorizPlane(pos - vec3(0.0, -2.0, 0.0)), 2));
    // m = mapMin(m, MapInfo(sdSphere(pos - vec3(3.0, 2.0, -0.5), 0.5), 3));

    //Box
    MapInfo m = MapInfo(sdInfHorizPlane(pos - vec3(0.0, -2.0, 0.0)), 1);
    m = mapMin(m, MapInfo(sdBox(pos - vec3(-5.0, 0.0, 0.0), vec3(0.25, 3.0, 6.0)), 4));
    m = mapMin(m, MapInfo(sdBox(pos - vec3( 5.0, 0.0, 0.0), vec3(0.25, 3.0, 6.0)), 2));
    m = mapMin(m, MapInfo(sdBox(pos - vec3( 0.0, 3.0, 0.0), vec3(5.0, 0.25, 6.0)), 1));
    m = mapMin(m, MapInfo(sdBox(pos - vec3( 0.0, 0.0, 4.0), vec3(5.0, 3.0, 0.25)), 1));
    m = mapMin(m, MapInfo(sdBox(pos - vec3( 0.0, 0.0,-6.0), vec3(5.0, 3.0, 0.25)), 1));

    //Objects in room
    m = mapMin(m, MapInfo(sdSphere(pos - vec3(-1.0, -1.0, 1.0), 1.0), 5));
    m = mapMin(m, MapInfo(sdSphere(pos - vec3(1.5, -0.7, 0.75), 0.75), 1));

    //Lights
    m = mapMin(m, MapInfo(sdBox(pos - vec3(0.0, 3.0, 0.0), vec3(1.0, 0.26, 1.0)), 3));

    return m;
}
