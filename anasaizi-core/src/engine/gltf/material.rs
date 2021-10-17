use crate::{
    engine::{gltf::mappers::ImportData, GLTFMaterial},
    math::{Matrix4, Vector3, Vector4},
    vulkan::ShaderFlags,
};

impl GLTFMaterial {
    pub(crate) fn from_gltf(g_material: gltf::Material<'_>, imp: &ImportData) -> Self {
        let pbr = g_material.pbr_metallic_roughness();

        let emission: Vector3 = g_material.emissive_factor().into();

        let mut alpha_cutoff = 0.0;
        let mut alpha_blend = 0.0;

        // alpha blending
        if g_material.alpha_mode() != gltf::material::AlphaMode::Opaque {
            alpha_blend = 1.0;

            if g_material.alpha_mode() == gltf::material::AlphaMode::Mask
                && g_material.alpha_cutoff().is_some()
            {
                alpha_cutoff = g_material.alpha_cutoff().unwrap();
            }
        }

        let mut material = GLTFMaterial {
            model_matrix: Matrix4::identity(),
            base_color_texture: -1,
            base_color_texture_coord: 0,
            base_color_factor: pbr.base_color_factor().into(),

            metallic_roughness_texture: -1,
            metallic_factor_texture_coord: 0,
            metallic_roughness_values: Vector4::new(
                pbr.metallic_factor(),
                pbr.roughness_factor(),
                0.0,
                0.0,
            ),

            normal_texture: -1,
            normal_texture_coord: 0,
            normal_scale: 0.0,

            occlusion_texture: -1,
            occlusion_texture_coord: 0,
            occlusion_strength: 0.0,

            emissive_texture: -1,
            emissive_texture_coord: 0,
            emissive_factor: Vector4::new(emission.x, emission.y, 1.0, 1.0),

            alpha_cutoff,
            alpha_mode: alpha_blend,
            scale_ibl_ambient: Vector4::identity(),
        };

        if let Some(color_info) = pbr.base_color_texture() {
            material.base_color_texture = load_texture(&color_info.texture(), imp);
            material.base_color_texture_coord = color_info.tex_coord() as i32;
        }
        if let Some(mr_info) = pbr.metallic_roughness_texture() {
            material.metallic_roughness_texture = load_texture(&mr_info.texture(), imp);
            material.metallic_factor_texture_coord = mr_info.tex_coord() as i32;
        }
        if let Some(normal_texture) = g_material.normal_texture() {
            material.normal_texture = load_texture(&normal_texture.texture(), imp);
            material.normal_scale = normal_texture.scale();
            material.normal_texture_coord = normal_texture.tex_coord() as i32;
        }
        if let Some(occ_texture) = g_material.occlusion_texture() {
            material.occlusion_texture = load_texture(&occ_texture.texture(), imp);
            material.occlusion_strength = occ_texture.strength();
            material.occlusion_texture_coord = occ_texture.tex_coord() as i32;
        }
        if let Some(em_info) = g_material.emissive_texture() {
            material.emissive_texture = load_texture(&em_info.texture(), imp);
            material.emissive_texture_coord = em_info.tex_coord() as i32;
        }

        material
    }

    pub fn shader_flags(&self) -> ShaderFlags {
        let mut flags = ShaderFlags::empty();
        if self.base_color_texture != -1 {
            flags |= ShaderFlags::HAS_BASECOLORMAP;
        }
        if self.normal_texture != -1 {
            flags |= ShaderFlags::HAS_NORMALMAP;
        }
        if self.emissive_texture != -1 {
            flags |= ShaderFlags::HAS_EMISSIVEMAP;
        }
        if self.metallic_roughness_texture != -1 {
            flags |= ShaderFlags::HAS_METALROUGHNESSMAP;
        }
        if self.occlusion_texture != -1 {
            flags |= ShaderFlags::HAS_OCCLUSIONMAP;
        }
        flags
    }
}

fn load_texture(g_texture: &gltf::texture::Texture<'_>, imp: &ImportData) -> i32 {
    let g_img = g_texture.source();

    let (id, _) = imp.texture_storage.at(g_img.index());

    *id as i32
}
