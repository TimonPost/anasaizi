#version 450
#extension GL_ARB_separate_shader_objects : enable

out gl_PerVertex {
    vec4 gl_Position;
};

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec3 inNormal;

layout(location = 0) out vec4 out_FragColor;
layout(location = 1) out vec2 out_FragTexCoord;
layout(location = 2) out vec3 out_FragPos;
layout(location = 3) out vec3 out_Normal;

layout(location = 4) out flat int out_albedoMap;
layout(location = 5) out flat int out_normalMap;
layout(location = 6) out flat int out_metallicMap;
layout(location = 7) out flat int out_roughnessMap;
layout(location = 8) out flat int out_aoMap;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform MeshPushConstants {
    mat4 model;
    int albedoMap;
    int normalMap;
    int metallicMap;
    int roughnessMap;
    int aoMap;
} push_constants;

void main() {
    gl_Position = ubo.proj * ubo.view * push_constants.model * vec4(inPosition, 1.0);

    out_FragColor = inColor;
    out_FragPos = vec3(push_constants.model * vec4(inPosition, 1.0));
    out_FragTexCoord = inTexCoord;
    out_Normal = mat3(transpose(inverse(push_constants.model))) * inNormal;

    out_albedoMap = push_constants.albedoMap;
    out_normalMap = push_constants.normalMap;
    out_metallicMap = push_constants.metallicMap;
    out_roughnessMap = push_constants.roughnessMap;
    out_aoMap = push_constants.aoMap;
}