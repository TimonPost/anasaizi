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

layout(binding = 4) uniform LightUniformBufferObject {
    float shininess;
    float specularStrenght;

    float ambientStrenght;

    vec3 lightPos;
    vec3 lightColor;

    vec3 viewPos;
} lightUbo;

void main() {
    vec3 objectColor = vec3(fragColor.rgb * texture(sampler2D(textures[imgIndex], samp), fragTexCoord).rgb);
    vec3 lightDir = normalize(lightUbo.lightPos - fragPos);

    /// Calculate ambient lighting.
    vec3 ambient = lightUbo.ambientStrenght * lightUbo.lightColor;

    /// Calculate the diffuse color.
    vec3 norm = normalize(normal);
    // 1. Find the light direction.

    // 2. Generate a value that becomes less as the angle becomes bigger.
    float diff = max(dot(norm, lightDir), 0.0);
    // 3. Dimm the light based up on this value.
    vec3 diffuse = diff * lightUbo.lightColor;

    /// Calculate specular lighting.
    vec3 viewDir = normalize(lightUbo.viewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);

    float spec = pow(max(dot(viewDir, reflectDir), 0.0), lightUbo.shininess);
    vec3 specular = lightUbo.specularStrenght * spec * lightUbo.lightColor;

    /// Add ambient and diffuse and use this factor for color brightness.
    vec3 result = (ambient + diffuse + specular) * objectColor;
    outColor =  vec4(result, 1.0);
}