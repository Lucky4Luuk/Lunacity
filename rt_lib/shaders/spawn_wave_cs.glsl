#version 450
#include "common.glsl"

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

//Input 1
layout(std430, binding = 0) buffer rayhit_input {
    RawRayHit ray_hit[];
};

//Input 2
layout(std430, binding = 1) buffer random_input {
    uint random_ssbo[2048];
};

//Output
layout(std430, binding = 2) buffer ray_buffer {
    RawRay ray_ssbo[];
};

uniform vec2 dims;
uniform float samples;

int flat_idx = int(dot(vec2(gl_GlobalInvocationID.xy), vec2(1, 4096)));
void encrypt_tea(inout uvec2 arg) {
	uvec4 key = uvec4(0xa341316c, 0xc8013ea4, 0xad90777d, 0x7e95761e);
	uint v0 = arg[0], v1 = arg[1];
	uint sum = 0u;
	uint delta = 0x9e3779b9u;

	for(int i = 0; i < 32; i++) {
		sum += delta;
		v0 += ((v1 << 4) + key[0]) ^ (v1 + sum) ^ ((v1 >> 5) + key[1]);
		v1 += ((v0 << 4) + key[2]) ^ (v0 + sum) ^ ((v0 >> 5) + key[3]);
	}
	arg[0] = v0;
	arg[1] = v1;
}

vec2 get_random(inout uint seed) {
  	uvec2 arg = uvec2(flat_idx, seed++);
  	encrypt_tea(arg);
  	return fract(vec2(arg) / vec2(0xffffffffu));
}

vec3 sampleHemisphere(vec3 normal, vec2 uv) {
    uv.x = 2.0 * uv.x - 1.0;
    float a = PI * 2.0 * uv.y;
    vec2 b = sqrt(1.0 - uv.x*uv.x) * vec2(cos(a), sin(a));
    return normalize(normal + vec3(b.x, b.y, uv.x));
}

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    uint random_index = (ray_index + uint(samples)) % 2048;

    RawRayHit rhit = ray_hit[ray_index];
    int objectID = int(rhit.pos_id.w);

    if (objectID > 0) {
        vec3 position = rhit.pos_id.xyz;
        vec3 normal = rhit.normal_dist.xyz;

        // vec2 rng = vec2(random_ssbo[random_index], random_ssbo[random_index2]);
        // uint seed = uint(samples);
        vec2 rng = get_random(random_ssbo[random_index]);
        vec3 newDir = sampleHemisphere(normal, rng);

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
