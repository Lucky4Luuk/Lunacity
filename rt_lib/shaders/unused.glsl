//File for storing currently unused snippets, that we will need in the future.

Ray rayFromProjview(vec2 uv) {
    vec2 pos = uv * 2.0 - 1.0;
    float near = 0.02;
    float far = 1024.0;
    vec3 origin = (invprojview * vec4(pos, -1.0, 1.0) * near).xyz;
    vec3 dir = (invprojview * vec4(pos * (far - near), far + near, far - near)).xyz;
    Ray ray;
    ray.pos = origin;
    ray.dir = normalize(dir);
    return ray;
}
