use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GltfPBRShaderConstants {
   pub has_basecolormap: u32,
   pub has_normalmap: u32,
   pub has_emissivemap: u32,
   pub has_metalroughnessmap: u32,
   pub has_occlusionmap: u32,
   pub use_ibl: u32,

   pub has_normals: u32,
   pub has_tangents: u32,
   pub has_colors: u32,
   pub has_uvs: u32,

   pub texture_array_lenght: u32,
}