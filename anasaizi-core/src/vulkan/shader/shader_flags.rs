use crate::engine::GltfPBRShaderConstants;
use bitflags::bitflags;
use std::fmt::Display;

bitflags! {
    /// Flags matching the defines in the PBR shader
    pub struct ShaderFlags: u16 {
        // vertex shader + fragment shader
        const HAS_NORMALS           = 1;
        const HAS_TANGENTS          = 1 << 1;
        const HAS_UV                = 1 << 2;
        const HAS_COLORS            = 1 << 3;

        // fragment shader only
        const USE_IBL               = 1 << 4;
        const HAS_BASECOLORMAP      = 1 << 5;
        const HAS_NORMALMAP         = 1 << 6;
        const HAS_EMISSIVEMAP       = 1 << 7;
        const HAS_METALROUGHNESSMAP = 1 << 8;
        const HAS_OCCLUSIONMAP      = 1 << 9;
        const USE_TEX_LOD           = 1 << 10;
    }
}

impl ShaderFlags {
    pub fn as_strings(self) -> Vec<String> {
        (0..15)
            .map(|i| 1u16 << i)
            .filter(|i| self.bits & i != 0)
            .map(|i| format!("{:?}", ShaderFlags::from_bits_truncate(i)))
            .collect()
    }
}

impl From<ShaderFlags> for GltfPBRShaderConstants {
    fn from(flags: ShaderFlags) -> Self {
        let mut constants = GltfPBRShaderConstants::default();

        if flags.contains(ShaderFlags::HAS_BASECOLORMAP) {
            constants.has_basecolormap = 1;
        }
        if flags.contains(ShaderFlags::HAS_EMISSIVEMAP) {
            constants.has_emissivemap = 1;
        }
        if flags.contains(ShaderFlags::HAS_METALROUGHNESSMAP) {
            constants.has_metalroughnessmap = 1;
        }
        if flags.contains(ShaderFlags::HAS_NORMALMAP) {
            constants.has_normalmap = 1;
        }
        if flags.contains(ShaderFlags::HAS_OCCLUSIONMAP) {
            constants.has_occlusionmap = 1;
        }

        if flags.contains(ShaderFlags::HAS_TANGENTS) {
            constants.has_tangents = 1;
        }
        if flags.contains(ShaderFlags::HAS_NORMALS) {
            constants.has_normals = 1;
        }
        if flags.contains(ShaderFlags::HAS_UV) {
            constants.has_uvs = 1;
        }
        if flags.contains(ShaderFlags::HAS_COLORS) {
            constants.has_colors = 1;
        }

        if flags.contains(ShaderFlags::USE_IBL) {
            constants.use_ibl = 1;
        }

        constants
    }
}

impl Display for ShaderFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "base: {} colors: {}, emisive: {}, roughness: {}, normal: {}, occlusion: {}, tangents: {}",
               self.contains(ShaderFlags::HAS_BASECOLORMAP),
               self.contains(ShaderFlags::HAS_COLORS),
               self.contains(ShaderFlags::HAS_EMISSIVEMAP),
               self.contains(ShaderFlags::HAS_METALROUGHNESSMAP),
               self.contains(ShaderFlags::HAS_NORMALMAP),
               self.contains(ShaderFlags::HAS_OCCLUSIONMAP),
               self.contains(ShaderFlags::HAS_TANGENTS)
        )
    }
}
