#version 450
#include "common.glsl"

layout(local_size_x = DISPATCH_SIZE_X, local_size_y = DISPATCH_SIZE_Y, local_size_z = 1) in;

//Input 1
layout(std430, binding = 0) buffer rayhit_input {
    RawRayHit ray_hit[];
};

//Input 2
layout(std430, binding = 1) buffer random_input {
    float random_ssbo[];
};

//Output
layout(std430, binding = 2) buffer ray_buffer {
    RawRay ray_ssbo[];
};

uniform vec2 dims;
uniform float samples;

float fast(vec2 v) {
    v = (1./4320.) * v + vec2(0.25,0.);
    float state = fract( dot( v * v, vec2(3571)));
    return fract( state * state * (3571. * 2.));
}

float hash1(inout float seed) {
    return fract(sin(seed += 0.1)*43758.5453123);
    // return fast(vec2(seed, seed++));
}

vec2 hash2(inout float seed) {
    return fract(sin(vec2(seed+=0.1,seed+=0.1))*vec2(43758.5453123,22578.1459123));
}

vec3 hash3(inout float seed) {
    return fract(sin(vec3(seed+=0.1,seed+=0.1,seed+=0.1))*vec3(43758.5453123,22578.1459123,19642.3490423));
}

vec3 sampleHemisphere(const vec3 n, inout float seed ) {
  	vec2 r = hash2(seed);

	vec3  uu = normalize( cross( n, vec3(0.0,1.0,1.0) ) );
	vec3  vv = cross( uu, n );

	float ra = sqrt(r.y);
	float rx = ra*cos(6.2831*r.x);
	float ry = ra*sin(6.2831*r.x);
	float rz = sqrt( 1.0-r.y );
	vec3  rr = vec3( rx*uu + ry*vv + rz*n );

    return normalize( rr );
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    uint random_index = (ray_index + uint(samples)) % uint(dims.x * dims.y);

    RawRayHit rhit = ray_hit[ray_index];
    int objectID = int(rhit.pos_id.w);

    if (objectID > 0) {
        vec3 position = rhit.pos_id.xyz;
        vec3 normal = rhit.normal_dist.xyz;

        // vec2 rng = vec2(random_ssbo[random_index], random_ssbo[random_index2]);
        // uint seed = uint(samples);
        vec3 newDir = sampleHemisphere(normal, random_ssbo[random_index]);

        float n_dot_l = dot(newDir, rhit.normal_dist.xyz);
        rhit.dir_pow.w = rhit.dir_pow.w * n_dot_l;

        vec3 albedo = vec3(1.0);
        if (objectID == 2) {
            albedo = vec3(1.0, 0.0, 0.0);
        } else if (objectID == 4) {
            albedo = vec3(0.0, 1.0, 0.0);
        }
        rhit.col_mask.rgb *= albedo;

        RawRay ray;
        ray.pos = vec4(position + normal * 0.05, 0.0);
        ray.dir = vec4(newDir, rhit.dir_pow.w);
        ray.pixel = rhit.pixel;
        ray.col_mask = rhit.col_mask;
        ray_ssbo[ray_index] = ray;
    } else {
        RawRay ray;
        ray.pos = vec4(0.0);
        //Setting this to 0.0 essentially eliminates the ray entirely, but
        //the ray will still attempt to take the maximum amount of steps.
        //Very bad for optimization, needs fixing.
        //TODO: Fix this
        ray.dir = vec4(0.0);
        ray.pixel = vec4(0.0);
        ray_ssbo[ray_index] = ray;
    }
}
