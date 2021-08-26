use crate::{
    engine::{
        camera::{Camera, CameraMovement},
        image::Texture,
        Event, VulkanApplication,
    },
    profile_fn,
    vulkan::{
        structures::SyncObjects, CommandBuffers, CommandPool, FrameBuffers, Queue, RenderPass,
        ShaderSet, SwapChain, UniformBufferObjectTemplate,
    },
};

use anasaizi_profile::profile;

use crate::{
    engine::{
        renderer::render_pipeline::RenderPipeline, BufferLayout, GpuMeshMemory, Layer,
        RenderContext, Transform, World,
    },
    math::PosOnlyVertex,
    model::{square_indices, square_vertices},
    reexports::imgui::{DrawCmd, DrawCmdParams, DrawData},
    utils::any_as_u8_slice,
    vulkan::{
        Application, IndexBuffer, Instance, LogicalDevice, MeshPushConstants, ObjectPicker,
        Pipeline, RenderPassBuilder, ShaderBuilder, ShaderIOBuilder, SubpassDescriptor,
        VertexBuffer, Window,
    },
};
use ash::{version::DeviceV1_0, vk};
use nalgebra::Vector4;
use std::{mem, mem::swap, ptr, time::Instant};
use winit::event::{ElementState, KeyboardInput, ModifiersState, MouseButton, VirtualKeyCode};

pub static FRAGMENT_SHADER: &str = "assets\\shaders\\build\\frag.spv";
pub static VERTEX_SHADER: &str = "assets\\shaders\\build\\vert.spv";
const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub fn create_sync_objects(device: &ash::Device) -> SyncObjects {
    let mut sync_objects = SyncObjects {
        image_available_semaphores: vec![],
        render_finished_semaphores: vec![],
        inflight_fences: vec![],
    };

    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::SemaphoreCreateFlags::empty(),
    };

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::FenceCreateFlags::SIGNALED,
    };

    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        unsafe {
            let image_available_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let render_finished_semaphore = device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Failed to create Semaphore Object!");
            let inflight_fence = device
                .create_fence(&fence_create_info, None)
                .expect("Failed to create Fence Object!");

            sync_objects
                .image_available_semaphores
                .push(image_available_semaphore);
            sync_objects
                .render_finished_semaphores
                .push(render_finished_semaphore);
            sync_objects.inflight_fences.push(inflight_fence);
        }
    }

    sync_objects
}

pub struct RenderLayer<U: UniformBufferObjectTemplate> {
    pub swapchain: SwapChain,
    pub render_pass: RenderPass,
    pub graphics_queue: Queue,
    present_queue: Queue,
    pub command_pool: CommandPool,

    frame_buffers: FrameBuffers,
    command_buffers: CommandBuffers,

    pub pipelines: Vec<Pipeline<U>>,

    pub ui_pipeline: Option<Pipeline<U>>,
    pub ui_mesh: *const GpuMeshMemory,
    pub ui_data: *const DrawData,

    pub texture_sampler: Option<vk::Sampler>,

    sync_object: SyncObjects,

    pub camera: Camera,
    pub last_y: f64,
    pub last_x: f64,

    pub current_frame: usize,
    delta_time: f32,

    pub world: World,
    mouse_down: bool,
    pub object_picker: ObjectPicker,
}

impl<U: UniformBufferObjectTemplate> Layer for RenderLayer<U> {
    fn initialize(&mut self, window: &Window, render_context: &RenderContext) {}

    fn on_event(&mut self, event: &Event) {
        match event {
            Event::MouseMove(position, modifiers) => {
                if modifiers.ctrl() {
                    let xoffset = position.x - self.last_x;
                    let yoffset = self.last_y - position.y;

                    self.last_x = position.x;
                    self.last_y = position.y;

                    self.camera.process_mouse(self.delta_time, xoffset, yoffset)
                }
            }
            Event::Keyboard(input) => {
                match (input.virtual_keycode, input.state) {
                    (Some(VirtualKeyCode::W), ElementState::Pressed) => {
                        self.camera
                            .process_movement(CameraMovement::FORWARD, self.delta_time);
                    }
                    (Some(VirtualKeyCode::A), ElementState::Pressed) => {
                        self.camera
                            .process_movement(CameraMovement::LEFT, self.delta_time);
                    }
                    (Some(VirtualKeyCode::S), ElementState::Pressed) => {
                        self.camera
                            .process_movement(CameraMovement::BACKWARD, self.delta_time);
                    }
                    (Some(VirtualKeyCode::D), ElementState::Pressed) => {
                        self.camera
                            .process_movement(CameraMovement::RIGHT, self.delta_time);
                    }
                    _ => {}
                };
            }
            Event::MouseScroll(_xoffset, yoffset) => {
                self.camera.process_mouse_scroll(*yoffset);
            }
            Event::MouseInput(state, button) => {
                if *button == MouseButton::Left && *state == ElementState::Pressed {
                    self.mouse_down = true;
                } else if *button == MouseButton::Left && *state == ElementState::Released {
                    self.mouse_down = false;
                }
            }
            _ => {}
        }
    }

    fn start_frame(&mut self) {}

    fn on_update(
        &mut self,
        delta_time: u128,
        render_context: &RenderContext,
        application: &VulkanApplication,
    ) {
        let device = render_context.device();
        let mut render_pipeline = RenderPipeline::<U>::new(
            &application.device,
            &self.command_buffers.current(),
            self.current_frame(),
        );

        let wait_fences = [self.sync_object.inflight_fences[self.current_frame()]];

        let (image_index, _is_sub_optimal) = unsafe {
            profile_fn!("Acquire Next Image...", {
                device
                    .reset_command_pool(*self.command_pool, vk::CommandPoolResetFlags::empty())
                    .unwrap();

                self.swapchain
                    .loader
                    .acquire_next_image(
                        self.swapchain.swapchain,
                        u64::MAX,
                        self.sync_object.image_available_semaphores[self.current_frame],
                        vk::Fence::null(),
                    )
                    .expect("Failed to acquire next image!")
            })
        };

        profile_fn!("Recording Commands...", {
            self.command_buffers.begin_session(
                &application.device,
                &self.render_pass,
                self.swapchain.extent,
                &self.frame_buffers,
                self.current_frame,
            );

            let mut render_pipeline = RenderPipeline::new(
                &application.device,
                &self.command_buffers.current(),
                self.current_frame(),
            );

            self.render_meshes(&mut render_pipeline);

            unsafe {
                device
                    .cmd_next_subpass(self.command_buffers.current(), vk::SubpassContents::INLINE);
            };

            self.render_ui(&mut render_pipeline);

            self.command_buffers.end_session(&application.device);
        });

        profile_fn!("Queues...", {
            let wait_semaphores = [self.sync_object.image_available_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores =
                [self.sync_object.render_finished_semaphores[self.current_frame]];

            let submit_infos = [vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: [self.command_buffers.current()].as_ptr(),
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
            }];

            unsafe {
                profile_fn!("Submitting Queues...", {
                    device
                        .reset_fences(&wait_fences)
                        .expect("Failed to reset Fence!");

                    device
                        .queue_submit(
                            *self.graphics_queue,
                            &submit_infos,
                            self.sync_object.inflight_fences[self.current_frame],
                        )
                        .expect("Failed to execute queue submit.");
                })
            }

            let swapchains = [*self.swapchain];

            let present_info = vk::PresentInfoKHR {
                s_type: vk::StructureType::PRESENT_INFO_KHR,
                p_next: ptr::null(),
                wait_semaphore_count: 1,
                p_wait_semaphores: signal_semaphores.as_ptr(),
                swapchain_count: 1,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: &image_index,
                p_results: ptr::null_mut(),
            };

            unsafe {
                profile_fn!("Present Queue...", {
                    let result = self
                        .swapchain
                        .loader
                        .queue_present(*self.present_queue, &present_info);

                    match result {
                        Err(vk::Result::ERROR_OUT_OF_DATE_KHR)
                        | Err(vk::Result::SUBOPTIMAL_KHR) => {
                            self.recreate_swapchain(application, render_context);
                        }
                        Err(_) => {
                            panic!("a");
                        }
                        _ => {}
                    }
                });

                device
                    .wait_for_fences(&wait_fences, true, u64::MAX)
                    .expect("Failed to wait for Fence!");
            }
        });

        // TODO: pick object
        if self.mouse_down {
            self.pick_object_pass(&render_context);
            self.mouse_down = false;
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    fn end_frame(&mut self) {}
}

impl<U: UniformBufferObjectTemplate> RenderLayer<U> {
    pub fn new(application: &VulkanApplication) -> Self {
        let device = &application.device;
        let instance = &application.instance;

        let graphics_queue = Queue::create(
            &device,
            device.queue_family_indices().graphics_family.unwrap(),
        );
        let present_queue = Queue::create(
            &device,
            device.queue_family_indices().present_family.unwrap(),
        );

        let command_pool = CommandPool::create(&device);

        let render_context = RenderContext::new(
            &application.instance,
            &command_pool,
            &device,
            &graphics_queue,
        );

        let swapchain = SwapChain::new(&render_context, application.window.surface_data());

        let render_pass = Self::setup_renderpass(swapchain.image_format, &application);

        let frame_buffers = FrameBuffers::create(
            &application.device,
            &render_pass,
            &swapchain.image_views.iter().map(|i| **i).collect(),
            swapchain.depth_image_view.clone(),
            &swapchain.extent,
        );

        let sync_object = create_sync_objects(device.logical_device());

        let camera = Camera::new(
            16.0 / 9.0,
            (swapchain.extent.width / swapchain.extent.height) as f32,
            0.1,
            100.0,
        );

        let texture_sampler = Texture::create_texture_sampler(&device);

        let command_buffers =
            CommandBuffers::create::<U>(&application.device, &command_pool, frame_buffers.len());

        let mut object_picker = ObjectPicker::new(
            application,
            &render_context,
            swapchain.extent.width as usize,
            swapchain.extent.height as usize,
        );

        RenderLayer {
            object_picker,
            swapchain,
            render_pass,

            command_pool,

            graphics_queue,
            present_queue,

            frame_buffers,
            command_buffers,
            pipelines: vec![],

            texture_sampler: Some(texture_sampler),

            sync_object,

            camera,

            last_x: 400.0,
            last_y: 300.0,
            current_frame: 0,
            ui_pipeline: None,
            ui_mesh: std::ptr::null(),
            ui_data: std::ptr::null(),
            delta_time: 0.0,

            world: World::new(),
            mouse_down: true,
        }
    }

    pub fn render_context(&self, application: &VulkanApplication) -> RenderContext {
        RenderContext::new(
            &application.instance,
            &self.command_pool,
            &application.device,
            &self.graphics_queue,
        )
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn create_pipeline(
        &mut self,
        application: &VulkanApplication,
        shader: ShaderSet<U>,
        pipeline_id: u32,
    ) {
        let mut pipeline = Pipeline::create(
            &application.device,
            self.swapchain.extent,
            &self.render_pass,
            shader,
            pipeline_id,
        );

        self.pipelines.push(pipeline);
    }

    pub fn pick_object_pass(&mut self, render_context: &RenderContext) {
        // self.object_picker.pick_object::<U>(self.last_x as usize, self.last_y as usize, self.last_key, render_context, &self.world);
    }

    pub fn render_meshes(&mut self, render_pipeline: &mut RenderPipeline<U>) {
        render_pipeline.set_view_port(
            0.0,
            0.0,
            self.swapchain.extent.width as f32,
            self.swapchain.extent.height as f32,
        );
        render_pipeline.set_scissors(
            0.0,
            0.0,
            self.swapchain.extent.width as f32,
            self.swapchain.extent.height as f32,
        );

        for pipeline in self.pipelines.iter() {
            render_pipeline.bind_pipeline(pipeline, &self.command_buffers);

            for (id, (mesh, transform, pipeline_id)) in self
                .world
                .query::<(&GpuMeshMemory, &Transform, &u32)>()
                .iter()
            {
                if *pipeline_id == pipeline.pipeline_id() {
                    // Push the model matrix using push constants.
                    let transform = MeshPushConstants {
                        model_matrix: transform.model_transform(),
                        texture_id: mesh.texture_id,
                    };

                    render_pipeline.set_mesh(mesh);
                    render_pipeline.push_mesh_constants(transform);
                    render_pipeline.render_mesh();
                }
            }
        }
    }

    pub fn render_ui(&mut self, render_pipeline: &mut RenderPipeline<U>) {
        if self.ui_data.is_null() || self.ui_mesh.is_null() {
            return;
        }

        if let Some(ui_pipeline) = &self.ui_pipeline {
            let draw_data = unsafe { &*self.ui_data };

            let framebuffer_width = draw_data.framebuffer_scale[0] * draw_data.display_size[0];
            let framebuffer_height = draw_data.framebuffer_scale[1] * draw_data.display_size[1];

            render_pipeline.bind_pipeline(ui_pipeline, &self.command_buffers);
            unsafe {
                render_pipeline.set_mesh(&*self.ui_mesh);
            }
            render_pipeline.set_view_port(0.0, 0.0, framebuffer_width, framebuffer_height);
            render_pipeline.push_ui_constants(draw_data);

            let index_offset = 0;
            let vertex_offset = 0;

            let clip_offset = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;

            for draw_list in draw_data.draw_lists() {
                for command in draw_list.commands() {
                    if let DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                vtx_offset,
                                idx_offset,
                                ..
                            },
                    } = command
                    {
                        unsafe {
                            let clip_x = (clip_rect[0] - clip_offset[0]) * clip_scale[0];
                            let clip_y = (clip_rect[1] - clip_offset[1]) * clip_scale[1];
                            let clip_w = (clip_rect[2] - clip_offset[0]) * clip_scale[0] - clip_x;
                            let clip_h = (clip_rect[3] - clip_offset[1]) * clip_scale[1] - clip_y;

                            render_pipeline.set_scissors(clip_x, clip_y, clip_w, clip_h);

                            render_pipeline.index_offset = index_offset + idx_offset as u32;
                            render_pipeline.vertex_offset = (vertex_offset + vtx_offset) as u32;
                            render_pipeline.index_count = count as u32;
                            render_pipeline.render_mesh();
                        }
                    }
                }
            }
        }
    }

    pub fn recreate_swapchain(
        &mut self,
        application: &VulkanApplication,
        render_context: &RenderContext,
    ) {
        unsafe {
            application
                .device
                .device_wait_idle()
                .expect("Failed to wait device idle!");

            self.destroy_swapchain(&application.device);
        };

        let surface_data = application.window.surface_data();

        self.swapchain = SwapChain::new(render_context, surface_data);

        self.render_pass = Self::setup_renderpass(self.swapchain.image_format, application);

        self.frame_buffers = FrameBuffers::create(
            &application.device,
            &self.render_pass,
            &self.swapchain.image_views.iter().map(|i| **i).collect(),
            self.swapchain.depth_image_view.clone(),
            &self.swapchain.extent,
        );
        self.command_buffers = CommandBuffers::create::<U>(
            &application.device,
            &self.command_pool,
            self.frame_buffers.len(),
        );

        unsafe {
            //self.ui_pipeline.as_mut().unwrap().refresh(&application.device, &self.swapchain, &self.render_pass);

            for pipeline in self.pipelines.iter_mut() {
                pipeline.refresh(&application.device, &self.swapchain, &self.render_pass);
            }
        }

        self.current_frame = 0;
    }

    fn setup_renderpass(format: vk::Format, application: &VulkanApplication) -> RenderPass {
        let mut dependecy = [vk::SubpassDependency::builder()
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .dependency_flags(vk::DependencyFlags::BY_REGION)
            .src_subpass(0)
            .dst_subpass(1)
            .build()];

        let render_pass = RenderPassBuilder::builder()
            .add_color_attachment(
                0,
                format,
                vk::ImageLayout::PRESENT_SRC_KHR,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            )
            .add_depth_attachment(
                1,
                application.device.find_depth_format(&application.instance),
            )
            .add_subpasses(
                vec![
                    SubpassDescriptor::new().with_color(0).with_depth(1),
                    SubpassDescriptor::new().with_color(0),
                ],
                &dependecy,
            )
            .build(&application.device);

        render_pass
    }

    unsafe fn destroy_swapchain(&self, device: &LogicalDevice) {
        self.command_buffers.free(device, &self.command_pool);
        self.frame_buffers.destroy(&device);
        self.render_pass.destroy(&device);
        self.swapchain.destroy(&device);
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe {
            self.destroy_swapchain(device);
            //self.ui_pipeline.as_ref().unwrap().destroy(&device);

            for pipeline in self.pipelines.iter() {
                pipeline.destroy(&device);
            }

            self.command_pool.destroy(&device);
            device.destroy_sampler(self.texture_sampler.unwrap(), None);
            self.sync_object.destroy(&device);
            self.ui_mesh.as_ref().unwrap().destroy(&device);
        }
    }

    pub fn grid_mesh(
        &self,
        application: &VulkanApplication,
        render_context: &RenderContext,
    ) -> (ShaderSet<U>, GpuMeshMemory) {
        let (square_vertices, square_indices) =
            (square_vertices().to_vec(), square_indices().to_vec());

        let grid_vertex_buffer =
            VertexBuffer::create::<PosOnlyVertex>(render_context, &square_vertices);

        let grid_index_buffer = IndexBuffer::create(render_context, &square_indices);

        let input_buffer_layout = BufferLayout::new().add_float_vec3(0);

        let push_const_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            offset: 0,
            size: mem::size_of::<MeshPushConstants>() as u32,
        }];

        let mut descriptors = ShaderIOBuilder::builder()
            .add_uniform_buffer(
                0,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            )
            .add_input_buffer_layout(input_buffer_layout)
            .add_push_constant_ranges(&push_const_ranges)
            .build(render_context, self.swapchain.images.len());

        let mut builder = ShaderBuilder::builder(
            application,
            "assets\\shaders\\build\\grid_vert.spv",
            "assets\\shaders\\build\\grid_frag.spv",
            self.swapchain.images.len(),
        )
        .with_descriptors(descriptors);

        let build: ShaderSet<U> = builder.build();

        (
            build,
            GpuMeshMemory::new(grid_vertex_buffer, grid_index_buffer, -1),
        )
    }
}

enum DrawCommand<'a> {
    ImGui(UiDrawElement<'a>),
    Mesh(),
}

struct UiDrawElement<'a> {
    mesh: &'a GpuMeshMemory,
    commands: Vec<DrawCommands>,
    /// Upper-left position of the viewport to render.
    ///
    /// (= upper-left corner of the orthogonal projection matrix to use)
    pub display_pos: [f32; 2],
    /// Size of the viewport to render.
    ///
    /// (= display_pos + display_size == lower-right corner of the orthogonal matrix to use)
    pub display_size: [f32; 2],
    /// Amount of pixels for each unit of display_size.
    ///
    /// Based on io.display_frame_buffer_scale. Typically [1.0, 1.0] on normal displays, and
    /// [2.0, 2.0] on Retina displays, but fractional values are also possible.
    pub framebuffer_scale: [f32; 2],
}

struct DrawCommands {
    pub count: usize,
    pub clip_rect: [f32; 4],
    pub vtx_offset: usize,
    pub idx_offset: usize,
}
