# Ideas
## RNG for low noise
Generate a buffer of random numbers, so each ray has its own random value generated on the CPU.

## Custom materials
Blender node-like system, that compiles to GLSL code, that gets included with #include.

## Combining ray waves
Per pixel, we send out 1 ray, and this goes for indirect rays too. This means if we want 32 random indirect samples, we will also send out 32 direct samples, and every ray only spawns 1 new ray.
If we simply take the result of each ray (for one pixel), add them together, and output them to a single texture, we have the final result. This means we can simply output each ray wave to the same texture, and keep adding the results, and it should work.
We can then use a texture to store the final output of all samples, and everytime we add a sample to it, we can simply calculate the final result as `current_total * (samples-1/samples) + new_sample * (1 / samples)`.
