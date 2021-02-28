#ifndef _INCLUDE_LAMBERT_
#define _INCLUDE_LAMBERT_

vec3 lambert(Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {
    float n_dot_l = dot(-light, normal);
    return mat.albedo * n_dot_l;
}

#endif
