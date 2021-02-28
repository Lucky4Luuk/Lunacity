//Raw rayhit for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRayHit {
    vec4 pos;
    vec4 normal_dist;
};

//Raw ray for sending through buffers. Vec4's are used instead of vec3's, because of alignment issues
//TODO: Better packing
struct RawRay {
    vec4 pos;
    vec4 dir;
};

struct Ray {
    vec3 pos;
    vec3 dir;
};

struct RayHit {
    vec3 pos;
    vec3 normal; //Surface normal
    float dist;
};
