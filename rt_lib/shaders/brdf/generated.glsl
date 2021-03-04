//THIS FILE WILL GET CLEARED EVERYTIME YOU RUN THE RAYTRACER
//ANY MODIFICATIONS HERE ARE POINTLESS
//YOU HAVE BEEN WARNED

#include "mat.glsl"
#include "lambert.glsl"
vec3 material(int id, Material mat, vec3 light, vec3 view, vec3 normal, vec3 tangent, vec3 binormal) {
	return builtin_lambert(mat, light, view, normal, tangent, binormal);
}