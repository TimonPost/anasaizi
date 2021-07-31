#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;

layout(location = 1) out vec3 nearPoint;
layout(location = 2) out vec3 farPoint;

// Shared set between most vertex shaders
layout(set = 0, binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

vec3 UnprojectPoint(float x, float y, float z, mat4 view, mat4 projection) {
    mat4 viewInv = inverse(view);
    mat4 projInv = inverse(projection);
    vec4 unprojectedPoint =  viewInv * projInv * vec4(x, y, z, 1.0);

    return unprojectedPoint.xyz / unprojectedPoint.w;

}

// normal vertice projection
void main() {
    vec3 p = inPosition;
    nearPoint = UnprojectPoint(p.x, p.y, 0.0, ubo.view, ubo.proj).xyz; // unprojecting on the near plane
    farPoint = UnprojectPoint(p.x, p.y, 1.0, ubo.view, ubo.proj).xyz; // unprojecting on the far plane

    gl_Position = vec4(p, 1.0); // using directly the clipped coordinates
}