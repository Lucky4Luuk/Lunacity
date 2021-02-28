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
    Color.rgb = clamp(Color.rgb, vec3(0.0), vec3(1.0)); //Clamp the range
    Color.rgb = pow(Color.rgb, vec3(1.0/2.2)); //Gamma correct (requires the clamp)
}
