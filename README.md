# Lunacity
A pathtracer written in Rust/OpenGL/GLSL, meant for offline rendering.

## TODO
- [ ] Workgroup size should be controllable from Rust (replace 32 by WORKGROUP_SIZE and let preprocessor change it)
- [ ] Scene support. Currently map() just contains a hardcoded scene, we need to load a scene in GLSL, so we can just generate the scene as a GLSL function, so we can let the compiler optimise it
- [ ] Mesh support (convert mesh to distant field)
- [ ] Material support (load GLSL, so that the UI could have a node system, or a code editor for materials)
