#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 in_fragColor;
layout(location = 1) in vec2 in_fragTexCoord;
layout(location = 2) in vec3 in_fragPos;
layout(location = 3) in vec3 in_normal;

layout(location = 4) in flat int in_albedoMap;
layout(location = 5) in flat int in_normalMap;
layout(location = 6) in flat int in_metallicMap;
layout(location = 7) in flat int in_roughnessMap;
layout(location = 8) in flat int in_aoMap;

layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform texture2D textures[8];

layout(binding = 3) uniform Light {
    vec4 position;
    vec4 viewPos;
    vec4 lightColor;
} light;

const float PI = 3.14159265359;

float DistributionGGX(vec3 N, vec3 H, float roughness);
float GeometrySchlickGGX(float NdotV, float roughness);
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness);
vec3 fresnelSchlick(float cosTheta, vec3 F0);
vec3 getNormalFromMap();

void main() {
    // Get values from material textures.
    vec3 albedo     = pow(texture(sampler2D(textures[in_albedoMap], samp), in_fragTexCoord).rgb, vec3(2.2));
    float metallic  = texture(sampler2D(textures[in_metallicMap], samp), in_fragTexCoord).r;
    float roughness = texture(sampler2D(textures[in_roughnessMap], samp), in_fragTexCoord).r;
    float ao        = texture(sampler2D(textures[in_albedoMap], samp), in_fragTexCoord).r;

    // Setup constants.
    vec3 N = getNormalFromMap();
    vec3 V = normalize(light.viewPos.xyz - in_fragPos);

    // Surface reflection at zero incidence
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    // reflectance equation
    vec3 Lo = vec3(0.0);

    for(int i = 0; i < 1; ++i)
    {
        // calculate per-light radiance
        vec3 L = normalize(light.position.xyz - in_fragPos);
        vec3 H = normalize(V + L);
        float distance    = length(light.position.xyz - in_fragPos);
        float attenuation = 1.0 / (distance * distance);
        vec3 radiance     =light.lightColor.xyz * attenuation;

        // cook-torrance brdf
        float NDF = DistributionGGX(N, H, roughness);
        float G   = GeometrySmith(N, V, L, roughness);
        vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

        // As the Fresnel value directly corresponds to kS we can use F to denote the specular contribution of any light that hits the surface. From kS we can then calculate the ratio of refraction kD:
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        vec3 numerator    = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        vec3 specular     = numerator / denominator;

        // add to outgoing radiance Lo
        float NdotL = max(dot(N, L), 0.0);
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    vec3 ambient = vec3(0.03) * albedo * ao;
    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0/2.2));

    outColor = vec4(color, 1.0);
}

vec3 getNormalFromMap()
{
    vec3 normal_map = texture(sampler2D(textures[in_normalMap], samp), in_fragTexCoord).xyz;
    vec3 tangentNormal = normal_map * 2.0 - 1.0;

    vec3 Q1  = dFdx(in_fragPos);
    vec3 Q2  = dFdy(in_fragPos);
    vec2 st1 = dFdx(in_fragTexCoord);
    vec2 st2 = dFdy(in_fragTexCoord);

    vec3 N   = normalize(in_normal);
    vec3 T  = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}

/// When the roughness is low (thus the surface is smooth), a highly concentrated number of microfacets are aligned to halfway vectors over a small radius.
float DistributionGGX(vec3 N, vec3 H, float roughness)
{
    float a      = roughness*roughness;
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float num   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

/// This is a formula for approximating the contribution of the Fresnel factor in the specular reflection of light from a non-conducting interface (surface) between two media.
vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(max(1.0 - cosTheta, 0.0), 5.0);
}

float GeometrySchlickGGX(float NdotV, float roughness)
{
    float r = (roughness + 1.0);
    float k = (r*r) / 8.0;

    float num   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

// To effectively approximate the geometry we need to take account of both the view direction (geometry obstruction) and the light direction vector (geometry shadowing).
float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2  = GeometrySchlickGGX(NdotV, roughness);
    float ggx1  = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}