use std::mem::size_of;
use nalgebra::{Vector2, Vector3};
use ash::vk;
use std::collections::HashMap;

#[derive(PartialOrd, PartialEq, Eq, Debug)]
pub struct BufferLayoutElement {
    pub stride: usize,
    pub format: vk::Format,
    pub layout_id: u8
}

impl BufferLayoutElement {
    pub fn new(layout_id: u8, stride: usize, format: vk::Format) -> BufferLayoutElement {
        BufferLayoutElement {
            stride,
            format,
            layout_id
        }
    }
}

pub struct BufferLayout {
    layouts: Vec<BufferLayoutElement>,
}

impl BufferLayout {
    pub fn new() -> BufferLayout {
        BufferLayout {
            layouts: Vec::new()
        }
    }

    pub fn add_float_vec2(mut self, layout_id: u8) -> Self {
        let stride = size_of::<Vector2<f32>>();
        self.layouts.push(BufferLayoutElement::new(layout_id, stride, vk::Format::R32G32_SFLOAT));
        self
    }

    pub fn add_float_vec3(mut self, layout_id: u8) -> Self {
        let stride = size_of::<Vector3<f32>>();
        self.layouts.push(BufferLayoutElement::new(layout_id, stride, vk::Format::R32G32B32_SFLOAT));
        self
    }

    pub fn get(&self, layout_id: u8) -> Option<&BufferLayoutElement> {
        for layout in self.layouts.iter() {
            if layout.layout_id == layout_id {
                return Some(&layout);
            }
        }
        None
    }

    pub fn build_binding_description(&self) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: self.layouts.iter().map(|x| x.stride).sum::<usize>() as u32,
            input_rate: vk::VertexInputRate::VERTEX
        }
    }

    pub fn build_attrib_description(&self) -> Vec<vk::VertexInputAttributeDescription> {
        let mut offset: usize = 0;
        let mut layouts = vec![];

        for layout in self.layouts.iter() {
            layouts.push(vk::VertexInputAttributeDescription {
                binding: 0,
                location: layout.layout_id as u32,
                format: layout.format,
                offset: offset as u32
            });

            offset += layout.stride;
        }

        layouts
    }
}

#[cfg(test)]
mod tests {
    use crate::vulkan::{BufferLayout};
    use crate::vulkan::buffer_layout::BufferLayoutElement;
    use ash::vk;

    #[test]
    fn add_float_vec2_correct_size() {
        let mut layout = BufferLayout::new()
            .add_float_vec2(0);

        assert_eq!(layout.get(0), Some(&BufferLayoutElement {
            layout_id: 0,
            stride: 2 * 4,
            format: vk::Format::R32G32_SFLOAT
        }));
    }

    #[test]
    fn add_float_vec3_correct_size() {
        let mut layout = BufferLayout::new()
            .add_float_vec3(0);

        assert_eq!(layout.get(0), Some(&BufferLayoutElement {
            layout_id: 0,
            stride: 3 * 4,
            format: vk::Format::R32G32B32_SFLOAT
        }));
    }

    #[test]
    fn build_attrib_description() {
        let mut buffer_layout = BufferLayout::new()
            .add_float_vec2(0)
            .add_float_vec3(1);

        let first_layout = buffer_layout.get(0).unwrap();
        let second_layout = buffer_layout.get(1).unwrap();

        let attrib_bindingen = buffer_layout.build_attrib_description();

        assert_eq!(attrib_bindingen[0].binding, 0);
        assert_eq!(attrib_bindingen[0].location,first_layout.layout_id as u32);
        assert_eq!(attrib_bindingen[0].format,first_layout.format);
        assert_eq!(attrib_bindingen[0].offset,0);

        assert_eq!(attrib_bindingen[1].binding, 0);
        assert_eq!(attrib_bindingen[1].location,second_layout.layout_id as u32);
        assert_eq!(attrib_bindingen[1].format,second_layout.format);
        assert_eq!(attrib_bindingen[1].offset,first_layout.stride  as u32);
    }
}
