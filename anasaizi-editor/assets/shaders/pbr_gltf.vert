// Originally taken from https://github.com/KhronosGroup/glTF-WebGL-PBR
// Commit a94655275e5e4e8ae580b1d95ce678b74ab87426

#version 450
#extension GL_ARB_separate_shader_objects : enable

/// ====== Constant variables ======

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

/// ====== IN variables ======

layout (location = 0) in vec4 a_Position;
layout (location = 2) in vec4 a_Normal;
layout (location = 4) in vec4 a_Tangent;
layout (location = 6) in vec2 a_UV_0; // TEXCOORD_0
layout (location = 7) in vec2 a_UV_1; // TEXCOORD_1
layout (location = 8) in vec4 a_Color; // COLOR_0

/// ====== OUT variables ======

layout(location=0) out vec4 v_Position;
layout(location=2) out vec2 v_UV[2];
layout(location=4) out vec4 v_Normal;
layout(location=6) out vec4 v_Color;
layout(location=8) out mat3 v_TBN;

/// ====== Uniform variables ======

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

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

void main()
{
    vec4 pos = pbr.model * a_Position;
    v_Position = vec4(vec3(pos.xyz) / pos.w, 1.0);

    if(HAS_NORMALS == 1) {
        if(HAS_TANGENTS == 1) {
            // TODO!: the reference shader was updated to use the normal matrix here
            vec3 normalW = normalize(vec3(pbr.model * vec4(a_Normal.xyz, 0.0)));
            vec3 tangentW = normalize(vec3(pbr.model * vec4(a_Tangent.xyz, 0.0)));
            vec3 bitangentW = cross(normalW, tangentW) * a_Tangent.w;
            v_TBN = mat3(tangentW, bitangentW, normalW);
        } else {
            v_Normal = normalize(pbr.model * vec4(a_Normal.xyz, 1.0));
        }
    }

    if(HAS_UV == 1) {
        v_UV[0] = a_UV_0;
        v_UV[1] = a_UV_1;
    } else {
        v_UV[0] = vec2(0., 0.);
        v_UV[1] = vec2(0., 0.);
    }

    if(HAS_COLORS == 1) {
        v_Color = a_Color;
    } else {
        v_Color = vec4(1.0);
    }

    gl_Position = ubo.proj * ubo.view * pbr.model * a_Position;
}


