#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (constant_id = 0) const int HAS_BASECOLORMAP = 0;
layout (constant_id = 1) const int HAS_NORMALMAP = 0;
layout (constant_id = 2) const int HAS_EMISSIVEMAP = 0;
layout (constant_id = 3) const int HAS_METALROUGHNESSMAP = 0;
layout (constant_id = 4) const int HAS_OCCLUSIONMAP = 0;
layout (constant_id = 5) const int USE_IBL = 0;
layout (constant_id = 6) const int HAS_NORMALS = 0;
layout (constant_id = 7) const int HAS_TANGENTS = 0;
layout (constant_id = 8) const int HAS_COLORS = 0;
layout (constant_id = 9) const int HAS_UV = 0;
layout (constant_id = 10) const int TEXTURE_ARRAY_LENGHT = 30;

layout(location=0) in vec4 v_Position;
layout(location=2) in vec2 v_UV[2];
layout(location=4) in vec4 v_Normal;
layout(location=6) in vec4 v_Color;
layout(location=8) in mat3 v_TBN;

layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform texture2D textures[TEXTURE_ARRAY_LENGHT];

layout(binding = 3) uniform Light {
    vec4 position;
    vec4 viewPos;
    vec4 lightColor;
    vec4 ambientLightColor;
    vec4 lightDirection;
    float ambientLightIntensity;
} light;

layout(push_constant) uniform MeshData {
    mat4 model;

    vec4 u_BaseColorFactor;
    vec4 u_MetallicRoughnessValues;
    vec4 u_EmissiveFactor;
    vec4 u_ScaleIBLAmbient;

    int u_BaseColorMap;
    int u_NormalMap;
    int u_MetallicRoughnessMap;
    int u_OcclusionMap;
    int u_EmissiveMap;

    int u_BaseColorTexCoord;
    int u_NormalTexCoord;
    int u_MetallicRoughnessTexCoord;
    int u_OcclusionTexCoord;
    int u_EmissiveTexCoord;

    float u_NormalScale;
    float u_OcclusionStrength;
    float u_AlphaCutoff;
    float u_AlphaBlend;
} pbr;

//uniform samplerCube u_DiffuseEnvSampler;
//uniform samplerCube u_SpecularEnvSampler;
//uniform sampler2D u_brdfLUT;

// Encapsulate the various inputs used by the various functions in the shading equation
// We store values in this struct to simplify the integration of alternative implementations
// of the shading terms, outlined in the Readme.MD Appendix.
struct PBRInfo
{
    float NdotL;                  // cos angle between normal and light direction
    float NdotV;                  // cos angle between normal and view direction
    float NdotH;                  // cos angle between normal and half vector
    float LdotH;                  // cos angle between light direction and half vector
    float VdotH;                  // cos angle between view direction and half vector
    float perceptualRoughness;    // roughness value, as authored by the model creator (input to shader)
    float metalness;              // metallic value at the surface
    vec3 reflectance0;            // full reflectance color (normal incidence angle)
    vec3 reflectance90;           // reflectance color at grazing angle
    float alphaRoughness;         // roughness mapped to a more linear change in the roughness (proposed by [2])
    vec3 diffuseColor;            // color contribution from diffuse lighting
    vec3 specularColor;           // color contribution from specular lighting
};

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;

vec3 get_texture(int map, vec2 coords) {
    return texture(sampler2D(textures[map], samp), coords).rgb;
}

// Find the normal for this fragment, pulling either from a predefined normal map
// or from the interpolated mesh normal and tangent attributes.
vec3 getNormal()
{
    mat3 tbn;
    vec3 n;

    // Retrieve the tangent space matrix
    if (HAS_TANGENTS == 1) {
        vec3 pos_dx = dFdx(v_Position.xyz);
        vec3 pos_dy = dFdy(v_Position.xyz);
        vec3 tex_dx = dFdx(vec3(v_UV[0], 0.0));
        vec3 tex_dy = dFdy(vec3(v_UV[0], 0.0));
        vec3 t = (tex_dy.t * pos_dx - tex_dx.t * pos_dy) / (tex_dx.s * tex_dy.t - tex_dy.s * tex_dx.t);

        vec3 ng;
        if(HAS_NORMALS == 1) {
            ng = normalize(v_Normal.xyz);
        } else {
            ng = cross(pos_dx, pos_dy);
        }

        t = normalize(t - ng * dot(ng, t));
        vec3 b = normalize(cross(ng, t));
        tbn = mat3(t, b, ng);
    } else {
        tbn = v_TBN;
    }

    if(HAS_NORMALMAP == 1) {
        n = get_texture(pbr.u_NormalTexCoord, v_UV[pbr.u_NormalMap]).rgb;
        n = normalize(tbn * ((2.0 * n - 1.0) * vec3(pbr.u_NormalScale, pbr.u_NormalScale, 1.0)));
    } else {
        // The tbn matrix is linearly interpolated, so we need to re-normalize
        n = normalize(tbn[2].xyz);
    }

    // reverse backface normals
    // TODO!: correct/best place? -> https://github.com/KhronosGroup/glTF-WebGL-PBR/issues/51
    n *= (2.0 * float(gl_FrontFacing) - 1.0);

    return n;
}


// Calculation of the lighting contribution from an optional Image Based Light source.
// Precomputed Environment Maps are required uniform inputs and are computed as outlined in [1].
// See our README.md on Environment Maps [3] for additional discussion.
vec3 getIBLContribution(PBRInfo pbrInputs, vec3 n, vec3 reflection)
{
    //    float mipCount = 9.0; // resolution of 512x512
    //    float lod = (pbrInputs.perceptualRoughness * mipCount);
    //    // retrieve a scale and bias to F0. See [1], Figure 3
    //    vec3 brdf = get_texture(u_brdfLUT, vec2(pbrInputs.NdotV, 1.0 - pbrInputs.perceptualRoughness)).rgb;
    //    vec3 diffuseLight = textureCube(u_DiffuseEnvSampler, n).rgb;
    //
    //    vec3 specularLight;
    //
    //    if(USE_TEX_LOD) {
    //        specularLight = textureCubeLodEXT(u_SpecularEnvSampler, reflection, lod).rgb;
    //    } else {
    //        specularLight = textureCube(u_SpecularEnvSampler, reflection).rgb;
    //    }
    //
    //    vec3 diffuse = diffuseLight * pbrInputs.diffuseColor;
    //    vec3 specular = specularLight * (pbrInputs.specularColor * brdf.x + brdf.y);
    //
    //    // For presentation, this allows us to disable IBL terms
    //    diffuse *= u_ScaleIBLAmbient.x;
    //    specular *= u_ScaleIBLAmbient.y;
    //
    //    return diffuse + specular;
    return vec3(0.0);
}

// Basic Lambertian diffuse
// Implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
// See also [1], Equation 1
vec3 diffuse(PBRInfo pbrInputs)
{
    return pbrInputs.diffuseColor / M_PI;
}

// The following equation models the Fresnel reflectance term of the spec equation (aka F())
// Implementation of fresnel from [4], Equation 15
vec3 specularReflection(PBRInfo pbrInputs)
{
    return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
}

// This calculates the specular geometric attenuation (aka G()),
// where rougher material will reflect less light back to the viewer.
// This implementation is based on [1] Equation 4, and we adopt their modifications to
// alphaRoughness as input as originally proposed in [2].
float geometricOcclusion(PBRInfo pbrInputs)
{
    float NdotL = pbrInputs.NdotL;
    float NdotV = pbrInputs.NdotV;
    float r = pbrInputs.alphaRoughness;

    float attenuationL = 2.0 * NdotL / (NdotL + sqrt(r * r + (1.0 - r * r) * (NdotL * NdotL)));
    float attenuationV = 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
    return attenuationL * attenuationV;
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// Implementation from "Average Irregularity Representation of a Roughened Surface for Ray Reflection" by T. S. Trowbridge, and K. P. Reitz
// Follows the distribution function recommended in the SIGGRAPH 2013 course notes from EPIC Games [1], Equation 3.
float microfacetDistribution(PBRInfo pbrInputs)
{
    float roughnessSq = pbrInputs.alphaRoughness * pbrInputs.alphaRoughness;
    float f = (pbrInputs.NdotH * roughnessSq - pbrInputs.NdotH) * pbrInputs.NdotH + 1.0;
    return roughnessSq / (M_PI * f * f);
}

layout(location = 0) out vec4 FragColor;

void main()
{
    // Metallic and Roughness material properties are packed together
    // In glTF, these factors can be specified by fixed scalar values
    // or from a metallic-roughness map
    float perceptualRoughness = pbr.u_MetallicRoughnessValues.y;
    float metallic = pbr.u_MetallicRoughnessValues.x;

    if(HAS_METALROUGHNESSMAP == 1) {
        // Roughness is stored in the 'g' channel, metallic is stored in the 'b' channel.
        // This layout intentionally reserves the 'r' channel for (optional) occlusion map data
        vec3 mrSample = get_texture(pbr.u_MetallicRoughnessMap, v_UV[pbr.u_MetallicRoughnessTexCoord]);
        perceptualRoughness = mrSample.g * perceptualRoughness;
        metallic = mrSample.b * metallic;
    }

    perceptualRoughness = clamp(perceptualRoughness, c_MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);

    // Roughness is authored as perceptual roughness; as is convention,
    // convert to material roughness by squaring the perceptual roughness [2].
    float alphaRoughness = perceptualRoughness * perceptualRoughness;

    vec4 baseColor;

    // The albedo may be defined from a base texture or a flat color
    if(HAS_BASECOLORMAP == 1) {
        baseColor = vec4(get_texture(pbr.u_BaseColorMap, v_UV[pbr.u_BaseColorTexCoord]), 1.0) * pbr.u_BaseColorFactor;
    } else {
        baseColor = pbr.u_BaseColorFactor;
    }

    // spec: COLOR_0 ... acts as an additional linear multiplier to baseColor
    baseColor *= v_Color;

    vec3 f0 = vec3(0.04);
    vec3 diffuseColor = baseColor.rgb * (vec3(1.0) - f0);
    diffuseColor *= 1.0 - metallic;
    vec3 specularColor = mix(f0, baseColor.rgb, metallic);

    // Compute reflectance.
    float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    vec3 specularEnvironmentR0 = specularColor.rgb;
    vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    vec3 n = getNormal();                             // normal at surface point
    vec3 v = normalize(light.viewPos.xyz - v_Position.xyz);        // Vector from surface point to camera
    vec3 l = normalize(light.lightDirection.xyz);             // Vector from surface point to light
    vec3 h = normalize(l+v);                          // Half vector between both l and v
    vec3 reflection = -normalize(reflect(v, n));

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
    float NdotV = clamp(abs(dot(n, v)), 0.001, 1.0);
    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float LdotH = clamp(dot(l, h), 0.0, 1.0);
    float VdotH = clamp(dot(v, h), 0.0, 1.0);

    PBRInfo pbrInputs = PBRInfo(
    NdotL,
    NdotV,
    NdotH,
    LdotH,
    VdotH,
    perceptualRoughness,
    metallic,
    specularEnvironmentR0,
    specularEnvironmentR90,
    alphaRoughness,
    diffuseColor,
    specularColor
    );

    // Calculate the shading terms for the microfacet specular shading model
    vec3 F = specularReflection(pbrInputs);
    float G = geometricOcclusion(pbrInputs);
    float D = microfacetDistribution(pbrInputs);

    // Calculation of analytical lighting contribution
    vec3 diffuseContrib = (1.0 - F) * diffuse(pbrInputs);
    vec3 specContrib = F * G * D / (4.0 * NdotL * NdotV);
    vec3 color = NdotL * light.lightColor.xyz * (diffuseContrib + specContrib);

    // Calculate lighting contribution from image based lighting source (IBL)
    if(USE_IBL == 1) {
        //color += getIBLContribution(pbrInputs, n, reflection);
    } else {
        // Add simple ambient light
        color += light.ambientLightColor.xyz * light.ambientLightIntensity * baseColor.xyz;
    }

    // Apply optional PBR terms for additional (optional) shading
    if(HAS_OCCLUSIONMAP == 1) {
        float ao = get_texture(pbr.u_OcclusionMap, v_UV[pbr.u_OcclusionTexCoord]).r;
        color = mix(color, color * ao, pbr.u_OcclusionStrength);
    }

    if(HAS_EMISSIVEMAP == 1) {
        vec3 emissive = get_texture(pbr.u_EmissiveMap, v_UV[pbr.u_EmissiveTexCoord]).rgb * pbr.u_EmissiveFactor.xyz;
        color += emissive;
    }

    // NOTE: the spec mandates to ignore any alpha value in 'OPAQUE' mode
    float alpha = mix(1.0, baseColor.a, pbr.u_AlphaBlend);
    if (pbr.u_AlphaCutoff > 0.0) {
        alpha = step(pbr.u_AlphaCutoff, baseColor.a);
    }

    if (alpha == 0.0) {
        discard;
    }

    FragColor = vec4(color, alpha);
}