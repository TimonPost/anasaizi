use crate::engine::gltf::mappers::ImportData;
use crate::math::{GltsVertex, Vector2, Vector4};
use nalgebra::Vector3;
use gltf::json::mesh::Mode;
use crate::engine::gltf::root::Root;
use std::path::Path;
use crate::engine::{PBRMeshPushConstants, RenderContext, GpuMeshMemory, Transform, GlTFPBRMeshPushConstants};
use crate::vulkan::ShaderFlags;

#[derive(Clone)]
pub struct Primitive {
    pub shader_flags: ShaderFlags,
    pub push_constants: GlTFPBRMeshPushConstants
}

impl Primitive {
    pub fn from_gltf(
        render_context: &mut RenderContext,
        g_primitive: &gltf::Primitive<'_>,
        primitive_index: usize,
        mesh_index: usize,
        root: &mut Root,
        imp: &ImportData,
        base_path: &Path,
        transform: Transform) -> Primitive
    {
        println!("{:?}", g_primitive.mode());

        let reader = g_primitive.reader(|buffer| Some(&imp.buffer_storage.at(buffer.index())));

        let mut shader_flags = ShaderFlags::empty();

        let positions = {
            let iter = reader
                .read_positions()
                .unwrap_or_else(||
                    panic!("primitives must have the POSITION attribute (mesh: {}, primitive: {})",
                           mesh_index, primitive_index)
                );
            iter.collect::<Vec<_>>()
        };

        let mut vertices: Vec<GltsVertex> = positions
            .into_iter()
            .map(|position| {
                GltsVertex {
                    position: Vector4::new(position[0], position[1], position[2], 1.0),
                    ..GltsVertex::default()
                }
            }).collect();

        // normals
        {
            if let Some(normals) = reader.read_normals() {
                for (i, normal) in normals.enumerate() {
                    vertices[i].normal = Vector4::new(normal[0], normal[1], normal[2], 1.0);
                }
                shader_flags |= ShaderFlags::HAS_NORMALS;
            } else {
                println!("Found no NORMALs for primitive {} of mesh {} \
                   (flat normal calculation not implemented yet)", primitive_index, mesh_index);
            }
        }

        // tangents
        {
            if let Some(tangents) = reader.read_tangents() {
                for (i, tangent) in tangents.enumerate() {
                    vertices[i].tangent = Vector4::from(tangent);
                }
                shader_flags |= ShaderFlags::HAS_TANGENTS;
            } else {
                println!("Found no TANGENTS for primitive {} of mesh {} \
                   (tangent calculation not implemented yet)", primitive_index, mesh_index);
            }
        }

        // texture coordinates
        {
            let mut tex_coord_set = 0;
            while let Some(tex_coords) = reader.read_tex_coords(tex_coord_set) {
                if tex_coord_set > 1 {
                    println!("Ignoring texture coordinate set {}, \
                        only supporting 2 sets at the moment. (mesh: {}, primitive: {})",
                          tex_coord_set, mesh_index, primitive_index);
                    tex_coord_set += 1;
                    continue;
                }
                for (i, tex_coord) in tex_coords.into_f32().enumerate() {
                    match tex_coord_set {
                        0 => vertices[i].tex_coord_0 = Vector2::from(tex_coord),
                        1 => vertices[i].tex_coord_1 = Vector2::from(tex_coord),
                        _ => unreachable!()
                    }
                }
                shader_flags |= ShaderFlags::HAS_UV;
                tex_coord_set += 1;
            }
        }

        // colors
        {
            if let Some(colors) = reader.read_colors(0) {
                let colors = colors.into_rgba_f32();
                for (i, c) in colors.enumerate() {
                    vertices[i].color_0 = c.into();
                }
                shader_flags |= ShaderFlags::HAS_COLORS;
            }
            if reader.read_colors(1).is_some() {
                println!("Ignoring further color attributes, only supporting COLOR_0. (mesh: {}, primitive: {})",
                      mesh_index, primitive_index);
            }
        }

        //TODO: joints and weights
        {
            if let Some(joints) = reader.read_joints(0) {
                for (i, joint) in joints.into_u16().enumerate() {
                   // vertices[i].joints_0 = joint;
                }
            }

            if reader.read_joints(1).is_some() {
                // println!("Ignoring further joint attributes, only supporting JOINTS_0. (mesh: {}, primitive: {})",
                //      mesh_index, primitive_index);
            }

            if let Some(weights) = reader.read_weights(0) {
                for (i, weights) in weights.into_f32().enumerate() {
                    //vertices[i].weights_0 = weights.into();
                }
            }
            if reader.read_weights(1).is_some() {
                //  println!("Ignoring further weight attributes, only supporting WEIGHTS_0. (mesh: {}, primitive: {})",
                //      mesh_index, primitive_index);
            }
        }

        let indices = reader
            .read_indices()
            .map(|read_indices| {
                read_indices.into_u32().collect::<Vec<_>>()
            }).unwrap();


        let mode = g_primitive.mode();

        if mode != Mode::Triangles {
            println!("only triangles are supported")
        }

        let g_material = g_primitive.material();

        let mut material: Option<GlTFPBRMeshPushConstants> = None;
        if let Some(mat) = root.materials
            .iter()
            .find(|(id, _)| *id == g_material.index().unwrap())
        {
            shader_flags |= mat.1.shader_flags();
            material = Some(mat.1.clone());
        }

        if material.is_none() {
            let mat = GlTFPBRMeshPushConstants::from_gltf(g_material.clone(), imp);
            root.materials.push((g_material.index().unwrap(), mat.clone()));
            shader_flags |= mat.shader_flags();
            material = Some(mat);
        };

        let mesh_memory = GpuMeshMemory::from_raw(render_context, vertices, indices, -1);
        let transform = transform;

        root.add_entity(shader_flags, mesh_memory, transform, material.as_ref().unwrap().clone());

        Primitive {
            shader_flags,
            push_constants: material.unwrap()
        }
    }
}