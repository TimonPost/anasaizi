
#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;
layout (location = 4) in vec3 in_tangent;
layout (location = 5) in vec3 in_bitangent;

layout(push_constant) uniform Matrices {
    mat4 ortho;
} matrices;

layout(location = 0) out vec4 outColor;
layout(location = 1) out vec2 outUV;

void main() {
    outColor = inColor;
    outUV = inTexCoord;

    gl_Position = matrices.ortho*vec4(inPosition.x, inPosition.y, 0.0, 1.0);
}
