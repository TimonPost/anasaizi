use std::mem::size_of;

use ash::vk;
use nalgebra::{Vector2, Vector3, Vector4};

/// Information about element in a buffer layout.
#[derive(PartialOrd, PartialEq, Eq, Debug)]
pub struct BufferLayoutElement {
    pub stride: usize,
    pub format: vk::Format,
    pub layout_id: u8,
}

impl BufferLayoutElement {
    pub fn new(layout_id: u8, stride: usize, format: vk::Format) -> BufferLayoutElement {
        BufferLayoutElement {
            stride,
            format,
            layout_id,
        }
    }
}

/// The layout of the data inside a buffer.
pub struct BufferLayout {
    layouts: Vec<BufferLayoutElement>,

    // those are used for pointers.
    pub binding_desc: Vec<vk::VertexInputBindingDescription>,
    pub attrib_desc: Vec<vk::VertexInputAttributeDescription>,
}

impl BufferLayout {
    pub fn new() -> BufferLayout {
        BufferLayout {
            layouts: Vec::new(),
            binding_desc: Vec::new(),
            attrib_desc: Vec::new(),
        }
    }

    /// Adds a 2 component float vector to the layout.
    pub fn add_float_vec2(mut self, layout_id: u8) -> Self {
        let stride = size_of::<Vector2<f32>>();
        self.layouts.push(BufferLayoutElement::new(
            layout_id,
            stride,
            vk::Format::R32G32_SFLOAT,
        ));
        self
    }

    /// Adds a 3 component float vector to the layout.
    pub fn add_float_vec3(mut self, layout_id: u8) -> Self {
        let stride = size_of::<Vector3<f32>>();
        self.layouts.push(BufferLayoutElement::new(
            layout_id,
            stride,
            vk::Format::R32G32B32_SFLOAT,
        ));
        self
    }

    /// Adds a 4 component float vector to the layout.
    pub fn add_float_vec4(mut self, layout_id: u8) -> Self {
        let stride = size_of::<Vector4<f32>>();
        self.layouts.push(BufferLayoutElement::new(
            layout_id,
            stride,
            vk::Format::R32G32B32A32_SFLOAT,
        ));
        self
    }

    /// Gets a layout element by the given id.
    pub fn get(&self, layout_id: u8) -> Option<&BufferLayoutElement> {
        for layout in self.layouts.iter() {
            if layout.layout_id == layout_id {
                return Some(&layout);
            }
        }
        None
    }

    /// Returns the binding descriptions that describe how a single buffer element is laid out in the buffer.
    pub fn build_binding_description(&mut self) {
        self.binding_desc.push(vk::VertexInputBindingDescription {
            binding: 0,
            stride: self.layouts.iter().map(|x| x.stride).sum::<usize>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        });
    }

    /// Returns the attribute descriptions that describes how a buffer element is structured.
    pub fn build_attrib_description(&mut self) {
        let mut offset: usize = 0;

        for layout in self.layouts.iter() {
            self.attrib_desc.push(vk::VertexInputAttributeDescription {
                binding: 0,
                location: layout.layout_id as u32,
                format: layout.format,
                offset: offset as u32,
            });

            offset += layout.stride;
        }
    }
}

#[cfg(test)]
mod tests {
    use ash::vk;

    use crate::engine::{buffer_layout::BufferLayoutElement, BufferLayout};

    #[test]
    fn add_float_vec2_correct_size() {
        let layout = BufferLayout::new().add_float_vec2(0);

        assert_eq!(
            layout.get(0),
            Some(&BufferLayoutElement {
                layout_id: 0,
                stride: 2 * 4,
                format: vk::Format::R32G32_SFLOAT
            })
        );
    }

    #[test]
    fn add_float_vec3_correct_size() {
        let layout = BufferLayout::new().add_float_vec3(0);

        assert_eq!(
            layout.get(0),
            Some(&BufferLayoutElement {
                layout_id: 0,
                stride: 3 * 4,
                format: vk::Format::R32G32B32_SFLOAT
            })
        );
    }

    #[test]
    fn build_attrib_description() {
        let mut buffer_layout = BufferLayout::new().add_float_vec2(0).add_float_vec3(1);
        buffer_layout.build_attrib_description();

        let first_layout = buffer_layout.get(0).unwrap();
        let second_layout = buffer_layout.get(1).unwrap();

        assert_eq!( buffer_layout.attrib_desc[0].binding, 0);
        assert_eq!( buffer_layout.attrib_desc[0].location, first_layout.layout_id as u32);
        assert_eq!( buffer_layout.attrib_desc[0].format, first_layout.format);
        assert_eq!( buffer_layout.attrib_desc[0].offset, 0);

        assert_eq!( buffer_layout.attrib_desc[1].binding, 0);
        assert_eq!( buffer_layout.attrib_desc[1].location, second_layout.layout_id as u32);
        assert_eq!( buffer_layout.attrib_desc[1].format, second_layout.format);
        assert_eq!( buffer_layout.attrib_desc[1].offset, first_layout.stride as u32);
    }
}
