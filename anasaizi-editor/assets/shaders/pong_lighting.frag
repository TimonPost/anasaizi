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
    float     shininess;
    int       diffuse;
    int       specular;
} material;

layout(binding = 5) uniform Light {
    vec4 position;

    vec4 lightAmbient;
    vec4 lightDiffuse;
    vec4 lightSpecular;

    vec4 viewPos;
    vec4 lightColor;
} light;

void main() {
    vec3 diffuse_texture = texture(sampler2D(textures[material.diffuse], samp), fragTexCoord).rgb;
    vec3 specular_texture = texture(sampler2D(textures[material.specular], samp), fragTexCoord).rgb;

    // difuse
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(light.position.xyz - fragPos);
    float diff = max(dot(norm, lightDir), 0.0);

    // specular
    vec3 viewDir = normalize(light.viewPos.xyz - fragPos);
    vec3 reflectDir = reflect(-lightDir, norm);

    float spec = pow(max(dot(viewDir, reflectDir), 0.0), material.shininess);

    vec3 ambient  = light.lightAmbient.xyz * diffuse_texture;
    vec3 diffuse = light.lightDiffuse.xyz * diff * diffuse_texture;
    vec3 specular = light.lightSpecular.xyz * spec * specular_texture;

    vec3 result = ambient + diffuse + specular;
    outColor = vec4(result, 1.0);

}