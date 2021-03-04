#ifndef _INCLUDE_LAMBERT_
#define _INCLUDE_LAMBERT_

vec3 builtin_lambert(Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {
    float n_dot_l = dot(light, normal);
    return mat.albedo * clamp(n_dot_l, 0.0, 1.0);
}

#endif
