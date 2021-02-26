#version 450 core

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 uv;
layout (location = 2) in vec4 Color;

out VS_OUTPUT {
    vec3 Color;
    vec2 UV;
} OUT;

void main()
{
    gl_Position = vec4(Position, 1.0);
    OUT.Color = Color.xyz; // ignore w component
    OUT.UV = uv;
}
