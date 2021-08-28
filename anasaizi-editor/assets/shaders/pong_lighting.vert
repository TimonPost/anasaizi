#version 450
#extension GL_ARB_separate_shader_objects : enable

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 0) out vec4 outFragColor;
layout(location = 1) out vec2 outFragTexCoord;
layout(location = 2) out flat int outImgIndex;
layout(location = 3) out vec3 outFragPos;
layout(location = 4) out vec3 outNormal;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform MeshPushConstants {
    mat4 model;
    int imgIndex;
} push_constants;

void main() {
    gl_Position = ubo.proj * ubo.view * push_constants.model * vec4(inPosition, 1.0);
    outFragPos = vec3(push_constants.model * vec4(inPosition, 1.0));

    outFragColor = inColor;
    outFragTexCoord = inTexCoord;
    outImgIndex = push_constants.imgIndex;
    outNormal = mat3(transpose(inverse(push_constants.model))) * inNormal;
}