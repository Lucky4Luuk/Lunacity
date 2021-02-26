#ifndef _INCLUDE_DISTANCE_FIELDS_
#define _INCLUDE_DISTANCE_FIELDS_

// Signed distance to a sphere.
// vec3 p  = ray position in model space
// float r = sphere radius
float sdSphere(vec3 p, float r) {
    return length(p) - r;
}

#endif
