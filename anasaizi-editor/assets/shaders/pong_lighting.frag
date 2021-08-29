#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) in flat int imgIndex;
layout(location = 3) in vec3 fragPos;
layout(location = 4) in vec3 normal;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform texture2D textures[2];

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(binding = 4) uniform Material {
    float shininess;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
} material;

layout(binding = 5) uniform Light {
    vec3 position;

    vec3 lightAmbient;
    vec3 lightDiffuse;
    vec3 lightSpecular;

    vec3 viewPos;
    vec3 lightColor;
} light;

void main() {
    // difuse
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(light.position - fragPos);
    float diff = max(dot(norm, lightDir), 0.0);

    // specular
    vec3 viewDir = normalize(light.viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);

    vec3 ambient  = light.lightAmbient * material.ambient;
    vec3 diffuse  = light.lightDiffuse * (diff * material.diffuse);
    vec3 specular = light.lightSpecular * (spec * material.specular);

    vec3 result = ambient + diffuse + specular;
    outColor = vec4(result, 1.0);

}