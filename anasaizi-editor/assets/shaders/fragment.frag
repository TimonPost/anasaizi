#version 450

#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in flat int imgIndex;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform texture2D textures[2];

void main() {
    outColor = vec4(fragColor.rgb * texture(sampler2D(textures[imgIndex], samp), fragTexCoord).rgb, 1.0);
}