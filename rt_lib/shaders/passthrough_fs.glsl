#version 450 core

uniform sampler2D tex;

in VS_OUTPUT {
    vec3 Color;
    vec2 UV;
} IN;

out vec4 Color;

void main()
{
    // Color = vec4(IN.UV, 0.0, 1.0);
    Color = texture(tex, IN.UV);
}
