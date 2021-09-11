use crate::engine::{RenderContext, UniformObjectTemplate, VulkanApplication, World};
use winit::event::KeyboardInput;

pub struct ObjectPicker {
    // dimensions: [u32; 2],
//
// // Tells the GPU where to write the color
// render_pass: RenderPass,
// pub pipeline: Pipeline,
//
// // Two attachments -> color and depth
// frame_buffers: FrameBuffers,
//
// // color attachment
// image: vk::Image,
// image_view: vk::ImageView,
// image_memory: vk::DeviceMemory,
//
// // depth attachment
// depth_image: vk::Image,
// depth_image_view: vk::ImageView,
// depth_image_memory: vk::DeviceMemory,
//
// buffer_memory: vk::DeviceMemory,
// buffer: vk::Buffer,
// buffer_size: u64,
// image_extend: vk::Extent2D,
}

impl ObjectPicker {
    pub fn new(
        _application: &VulkanApplication,
        _render_context: &RenderContext,
        _width: usize,
        _height: usize,
    ) -> ObjectPicker {
        // let input_buffer_layout = BufferLayout::new().add_float_vec3(0);
        //
        // let push_const_ranges = [vk::PushConstantRange {
        //     stage_flags: vk::ShaderStageFlags::VERTEX,
        //     offset: 0,
        //     size: size_of::<ObjectIdPushConstants>() as u32,
        // }];
        //
        // let descriptors = ShaderIOBuilder::builder()
        //     .add_input_buffer_layout(input_buffer_layout)
        //     .add_push_constant_ranges(&push_const_ranges)
        //     .add_uniform_buffer(0, vk::ShaderStageFlags::VERTEX)
        //     .build::<UniformBufferObject>(render_context, 1);
        //
        // let mut builder = ShaderBuilder::builder(
        //     application,
        //     "assets/shaders/build/mouse_pick_vert.spv",
        //     "assets/shaders/build/mouse_pick_frag.spv",
        //     1,
        // )
        // .with_descriptors(descriptors);
        // let mut shader = builder.build();
        //
        // let image_extend = vk::Extent2D {
        //     width: width as u32,
        //     height: height as u32,
        // };
        //
        // let (image, image_memory) = Texture::create_image(
        //     render_context,
        //     width as u32,
        //     height as u32,
        //     vk::Format::B8G8R8A8_SRGB,
        //     vk::ImageTiling::OPTIMAL,
        //     vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::COLOR_ATTACHMENT,
        //     vk::MemoryPropertyFlags::DEVICE_LOCAL,
        // );
        //
        // let image_view = ImageView::create(
        //     render_context.device(),
        //     image,
        //     vk::Format::B8G8R8A8_SRGB,
        //     vk::ImageAspectFlags::COLOR,
        // );
        //
        // let (depth_image, depth_image_view, depth_image_memory) =
        //     SwapChain::create_depth_resources(render_context, image_extend);
        //
        // let mut dependecies = [
        //     vk::SubpassDependency::builder()
        //         .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        //         .dst_stage_mask(vk::PipelineStageFlags::TRANSFER)
        //         .src_access_mask(
        //             vk::AccessFlags::COLOR_ATTACHMENT_WRITE
        //                 | vk::AccessFlags::COLOR_ATTACHMENT_READ,
        //         )
        //         .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
        //         .src_subpass(0)
        //         .dst_subpass(vk::SUBPASS_EXTERNAL)
        //         .dependency_flags(vk::DependencyFlags::VIEW_LOCAL_KHR)
        //         .build(),
        //     vk::SubpassDependency::builder()
        //         .src_stage_mask(vk::PipelineStageFlags::TRANSFER)
        //         .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        //         .src_access_mask(vk::AccessFlags::TRANSFER_READ)
        //         .dst_access_mask(
        //             vk::AccessFlags::COLOR_ATTACHMENT_WRITE
        //                 | vk::AccessFlags::COLOR_ATTACHMENT_READ,
        //         )
        //         .src_subpass(vk::SUBPASS_EXTERNAL)
        //         .dst_subpass(0)
        //         .dependency_flags(vk::DependencyFlags::VIEW_LOCAL_KHR)
        //         .build(),
        //     vk::SubpassDependency::builder()
        //         .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
        //         .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        //         .dst_access_mask(
        //             vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
        //                 | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        //         )
        //         .src_subpass(vk::SUBPASS_EXTERNAL)
        //         .dst_subpass(0)
        //         .dependency_flags(vk::DependencyFlags::BY_REGION)
        //         .build(),
        //     vk::SubpassDependency::builder()
        //         .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
        //         .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
        //         .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
        //         .src_subpass(0)
        //         .dst_subpass(vk::SUBPASS_EXTERNAL)
        //         .dependency_flags(vk::DependencyFlags::BY_REGION)
        //         .build(),
        // ];
        //
        // let render_pass = RenderPassBuilder::builder()
        //     .add_color_attachment(
        //         0,
        //         vk::Format::B8G8R8A8_SRGB,
        //         vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        //         vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        //     )
        //     .add_depth_attachment(
        //         1,
        //         application.device.find_depth_format(&application.instance),
        //     )
        //     .add_subpasses(
        //         vec![SubpassDescriptor::new().with_color(0).with_depth(1)],
        //         &dependecies,
        //     )
        //     .build(&application.device);
        //
        // let pipeline = Pipeline::object_pick_pipeline(
        //     &application.device,
        //     &render_pass,
        //     shader,
        //     &image_extend,
        // );
        //
        // let frame_buffers = FrameBuffers::create(
        //     &application.device,
        //     &render_pass,
        //     &vec![*image_view.clone()],
        //     depth_image_view.clone(),
        //     &image_extend,
        // );
        //
        // let image_size = (std::mem::size_of::<u8>() as u32 * (width * height * 4_usize) as u32)
        //     as vk::DeviceSize;
        //
        // let buffer = create_allocate_vk_buffer(
        //     render_context,
        //     image_size,
        //     vk::BufferUsageFlags::all(),
        //     vk::MemoryPropertyFlags::HOST_VISIBLE
        //         | vk::MemoryPropertyFlags::HOST_COHERENT
        //         | vk::MemoryPropertyFlags::HOST_CACHED,
        // );

        ObjectPicker {
            // frame_buffers,
            // render_pass,
            // pipeline,
            //
            // image,
            // image_view: *image_view,
            // image_memory,
            //
            // depth_image,
            // depth_image_view: *depth_image_view,
            // depth_image_memory,
            //
            // dimensions: [width as u32, height as u32],
            // buffer: buffer.0,
            // buffer_memory: buffer.1,
            //
            // buffer_size: image_size,
            // image_extend,
        }
    }

    // fn create_pushconstants(id: usize, model_matrix: Matrix4<f32>) -> ObjectIdPushConstants {
    //     ObjectIdPushConstants {
    //         color: Vector4::new(
    //             ((id & 0xFF) as f32) / 255.0,
    //             ((id >> 8) & 0xFF) as f32 / 255.0,
    //             ((id >> 16) & 0xFF) as f32 / 255.0,
    //             1.0,
    //         ), // Transparent means no entity.,
    //         model_matrix,
    //     }
    // }
    //
    // /// Get the ID from a RGBA value. transparent means None
    // fn get_entity_id(r: u8, g: u8, b: u8, a: u8) -> Option<usize> {
    //     if a == 0 {
    //         None
    //     } else {
    //         Some((r as usize) | (g as usize) << 8 | (b as usize) << 16)
    //     }
    // }

    /// Return either ID of picked object or None if did not click on anything
    pub fn pick_object<U: UniformObjectTemplate>(
        &mut self,
        _x: usize,
        _y: usize,
        _last_key: KeyboardInput,
        _render_context: &RenderContext,
        _world: &mut World,
    ) {

        // let command_buffer = begin_single_time_command(render_context);
        //
        // let mut command_buffers = CommandBuffers::from(command_buffer);
        // let command_buffer = command_buffers.get(0);
        //
        // let mut render_pipeline = RenderPipeline::new(render_context.logical_device(), &command_buffer, 0);
        //
        // let clear_values = [
        //     vk::ClearValue {
        //         color: vk::ClearColorValue {
        //             float32: [0.1, 0.1, 0.1, 1.0],
        //         },
        //     },
        //     vk::ClearValue {
        //         depth_stencil: vk::ClearDepthStencilValue {
        //             depth: 1.0,
        //             stencil: 0,
        //         },
        //     },
        // ];
        //
        // let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
        //     .render_area(vk::Rect2D {
        //         offset: vk::Offset2D { x: 0, y: 0 },
        //         extent: self.image_extend,
        //     })
        //     .framebuffer(self.frame_buffers.get(0))
        //     .clear_values(&clear_values)
        //     .render_pass(*self.render_pass)
        //     .build();
        // unsafe {
        //     render_context.device().cmd_begin_render_pass(
        //         command_buffer,
        //         &render_pass_begin_info,
        //         vk::SubpassContents::INLINE,
        //     );
        // }
        //
        // render_pipeline.set_view_port(
        //     0.0,
        //     0.0,
        //     self.image_extend.width as f32,
        //     self.image_extend.height as f32,
        // );
        // render_pipeline.set_scissors(
        //     0.0,
        //     0.0,
        //     self.image_extend.width as f32,
        //     self.image_extend.height as f32,
        // );
        //
        // render_pipeline.bind_pipeline(&self.pipeline, &command_buffers);
        //
        // for (id, (mesh, transform)) in world.query::<(&GpuMeshMemory, &Transform)>().iter() {
        //     if id.id() != 2 {
        //         render_pipeline.set_mesh(mesh);
        //         render_pipeline.push_mesh_constants::<ObjectIdPushConstants>(Self::create_pushconstants(id.id() as usize, transform.model_transform()));
        //         render_pipeline.render_mesh();
        //     }
        // }
        //
        // unsafe { render_context.device().cmd_end_render_pass(command_buffer); }
        //
        // copy_image_to_buffer(render_context, self.image, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, self.buffer, command_buffer, self.buffer_size, self.image_extend);
        //
        // end_single_time_command(render_context, &command_buffer);
        //
        // let mut buffer: Vec<u8> = Vec::with_capacity(self.buffer_size as usize);
        // unsafe  {
        //     let data_ptr = render_context.device()
        //         .map_memory(self.buffer_memory, 0, self.buffer_size, vk::MemoryMapFlags::all())
        //         .expect("Could not copy image memory to application buffer.");
        //
        //     data_ptr.copy_to_nonoverlapping(buffer.as_mut_ptr() as *mut c_void, self.buffer_size as usize);
        //
        //     render_context.device().unmap_memory(self.buffer_memory);
        // }
        // let buf_pos = 4 * (y * (self.dimensions[0] as usize) + x);
    }
}
