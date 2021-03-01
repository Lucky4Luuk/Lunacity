#version 450

layout(local_size_x = 32, local_size_y = 32, local_size_z = 1) in;

layout(rgba32f, binding = 0) uniform image2D img_input; //New sample
layout(rgba32f, binding = 1) uniform image2D img_output; //Current total

uniform int samples;

void main() {
    vec3 current = imageLoad(img_output, ivec2(gl_GlobalInvocationID.xy)).rgb;
    vec3 new = imageLoad(img_input, ivec2(gl_GlobalInvocationID.xy)).rgb;

    float samplesF = float(samples);
    vec3 final = current * ((samplesF-1.0)/samplesF) + new * (1.0 / samplesF);
    // vec3 final = vec3(samplesF/2.0);

    imageStore(img_output, ivec2(gl_GlobalInvocationID.xy), vec4(final, 1.0));
}
