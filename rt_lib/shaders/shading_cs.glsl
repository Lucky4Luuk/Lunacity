#version 450
#include "common.glsl"

layout(local_size_x = DISPATCH_SIZE_X, local_size_y = DISPATCH_SIZE_Y, local_size_z = 1) in;
layout(rgba32f, binding = 0) uniform image2D img_output;

layout(std430, binding = 1) buffer rayhit_input {
    RawRayHit ray_hit[];
};

uniform vec2 dims;

#include "brdf/mat.glsl"
#include "brdf/lambert.glsl"

void main() {
    uint ray_index = gl_GlobalInvocationID.x + gl_GlobalInvocationID.y * uint(dims.x);
    RawRayHit rhit = ray_hit[ray_index];

    int objectID = int(rhit.pos_id.w);

    ivec2 pixel_coords = ivec2(rhit.pixel.xy);

    //Get the current pixel. This will be (0,0,0) for the first raywave, and the total of all previous raywaves otherwise.
    vec3 current = imageLoad(img_output, pixel_coords).rgb;
    vec3 final = vec3(0.0);

    //TODO: Keep track of colour mask so that bounced light is correct

    if (objectID == 0) { //Ray hit the sky
        // final = vec3(1.0); //Skybox colour
    } else { //Ray hit another object
        //We don't care about the lighting bouncing off this object
        //to the current point we are shading, because this is
        //already handled by the hit on that object.

        if (objectID == 3) {
            final = vec3(1.0) * 5.0 * rhit.dir_pow.w * rhit.col_mask.rgb;
        }
    }

    //Store the current + final, because to combine all the raywaves, we simply need to add the result together.
    //Every ray only spawns 1 new ray, so we can always just add them together.
    imageStore(img_output, pixel_coords, vec4(current + final, 1.0));
}
