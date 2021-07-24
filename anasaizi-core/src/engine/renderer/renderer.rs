use crate::{
    debug::*,
    engine::{
        camera::{Camera, CameraMovement},
        image::Texture,
        Event, VulkanApplication,
    },
    model::Mesh,
    profile_fn,
    vulkan::{
        structures::SyncObjects, BufferLayout, CommandBuffers, CommandPool, FrameBuffers,
        LogicalDevice, Queue, RenderPass, ShaderSet, SwapChain, UniformBufferObjectTemplate,
    },
};

use anasaizi_profile::profile;

use crate::{
    math::PosOnlyVertex,
    model::{square_indices, square_vertices},
    vulkan::{
        Application, IndexBuffer, Pipeline, ShaderBuilder, UniformBufferObject, VertexBuffer,
    },
};
use ash::{version::DeviceV1_0, vk};
use std::{path::Path, ptr, time::Instant};
use winit::event::{ElementState, VirtualKeyCode};

pub static FRAGMENT_SHADER: &str = "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\frag.spv";
pub static VERTEX_SHADER: &str = "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\vert.spv";
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

pub struct RenderObject<U: UniformBufferObjectTemplate> {
    pub pipeline: Pipeline,
    pub mesh: Mesh,
    pub shader: ShaderSet<U>,
}

pub struct VulkanRenderer<U: UniformBufferObjectTemplate> {
    swapchain: SwapChain,
    render_pass: RenderPass,
    pub graphics_queue: Queue,
    present_queue: Queue,
    pub command_pool: CommandPool,

    frame_buffers: Option<FrameBuffers>,
    buffers: Option<CommandBuffers>,
    pub texture_sampler: Option<vk::Sampler>,
    pub render_objects: Vec<RenderObject<U>>,

    sync_object: SyncObjects,

    pub current_frame: usize,
    pub last_frame: Instant,
    pub delta_time: f32,
    pub start_time: Instant,

    pub camera: Camera,
    pub last_y: f64,
    pub last_x: f64,
}

impl<U: UniformBufferObjectTemplate> VulkanRenderer<U> {
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

        let swapchain = SwapChain::new(&instance, &device, application.window.surface_data());

        let render_pass = RenderPass::create(&instance, &device, swapchain.image_format);

        let sync_object = create_sync_objects(device.logical_device());

        let camera = Camera::new(
            16.0 / 9.0,
            (swapchain.extent.width / swapchain.extent.height) as f32,
            0.1,
            100.0,
        );

        let texture_sampler = Texture::create_texture_sampler(&device);

        VulkanRenderer {
            swapchain,
            render_pass,

            command_pool,

            graphics_queue,
            present_queue,

            frame_buffers: None,
            buffers: None,
            render_objects: vec![],
            texture_sampler: Some(texture_sampler),

            sync_object,

            camera,

            delta_time: 0.0,

            last_frame: Instant::now(),
            start_time: Instant::now(),
            current_frame: 0,

            last_x: 400.0,
            last_y: 300.0,
        }
    }

    pub fn camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn push_render_object(
        &mut self,
        application: &VulkanApplication,
        shader: ShaderSet<U>,
        mesh: Mesh,
    ) {
        let pipeline = Pipeline::create(
            &application.device,
            self.swapchain.extent,
            &self.render_pass,
            &shader,
        );

        self.render_objects.push(RenderObject {
            pipeline,
            shader,
            mesh,
        });

        let frame_buffers = FrameBuffers::create(
            &application.device,
            &self.render_pass,
            &self.swapchain.image_views.iter().map(|i| **i).collect(),
            self.swapchain.depth_image_view,
            &self.swapchain.extent,
        );

        self.render_objects.push(self.load_grid_mesh(application));

        let buffers = CommandBuffers::create(
            &application.device,
            &self.command_pool,
            &self.render_objects,
            &frame_buffers,
            &self.render_pass,
            self.swapchain.extent,
        );

        self.buffers = Some(buffers);
        self.frame_buffers = Some(frame_buffers);
    }

    pub fn initialize_resources() {}

    pub fn start_frame(&mut self) {
        let current_frame = Instant::now();

        self.delta_time = (current_frame - self.last_frame).as_millis() as f32;
        self.last_frame = current_frame;
    }

    #[profile(VulkanRenderer)]
    pub fn draw(&mut self, application: &VulkanApplication) {
        let device = &application.device;

        let wait_fences = [self.sync_object.inflight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            profile_fn!("Acquire Next Image...", {
                device
                    .wait_for_fences(&wait_fences, true, u64::MAX)
                    .expect("Failed to wait for Fence!");

                self.swapchain
                    .loader
                    .acquire_next_image(
                        self.swapchain.swapchain,
                        u64::MAX,
                        self.sync_object.image_available_semaphores[self.current_frame],
                        vk::Fence::null(),
                    )
                    .expect("Failed to acquire next image.")
            })
        };

        profile_fn!("Queues...", {
            let wait_semaphores = [self.sync_object.image_available_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores =
                [self.sync_object.render_finished_semaphores[self.current_frame]];

            let command_buffer = self.buffers.as_ref().unwrap();

            let submit_infos = [vk::SubmitInfo {
                s_type: vk::StructureType::SUBMIT_INFO,
                p_next: ptr::null(),
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: 1,
                p_command_buffers: [command_buffer.get(image_index as usize)].as_ptr(),
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
                    self.swapchain
                        .loader
                        .queue_present(*self.present_queue, &present_info)
                        .expect("Failed to execute queue present.");
                });
            }
        });

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn end_frame() {}

    pub fn clean_resources() {}

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::MouseMove(position) => {
                let xoffset = position.x - self.last_x;
                let yoffset = self.last_y - position.y;

                self.last_x = position.x;
                self.last_y = position.y;

                self.camera.process_mouse(self.delta_time, xoffset, yoffset)
            }
            Event::Keyboard(input) => match (input.virtual_keycode, input.state) {
                (Some(VirtualKeyCode::W), ElementState::Pressed) => {
                    self.camera
                        .process_key(CameraMovement::FORWARD, self.delta_time);
                }
                (Some(VirtualKeyCode::A), ElementState::Pressed) => {
                    self.camera
                        .process_key(CameraMovement::LEFT, self.delta_time);
                }
                (Some(VirtualKeyCode::S), ElementState::Pressed) => {
                    self.camera
                        .process_key(CameraMovement::BACKWARD, self.delta_time);
                }
                (Some(VirtualKeyCode::D), ElementState::Pressed) => {
                    self.camera
                        .process_key(CameraMovement::RIGHT, self.delta_time);
                }
                _ => {}
            },
            Event::MouseScroll(_xoffset, yoffset) => {
                self.camera.process_mouse_scroll(yoffset);
            }
            _ => {}
        }
    }

    fn load_grid_mesh(&self, application: &VulkanApplication) -> RenderObject<U> {
        let (square_vertices, square_indices) =
            (square_vertices().to_vec(), square_indices().to_vec());

        let grid_vertex_buffer = VertexBuffer::create::<PosOnlyVertex>(
            &application.instance,
            &application.device,
            &square_vertices,
            &self.graphics_queue,
            &self.command_pool,
        );

        let grid_index_buffer = IndexBuffer::create(
            &application.instance,
            &application.device,
            &square_indices,
            &self.graphics_queue,
            &self.command_pool,
        );

        let mut descriptor_write_sets = vec![vk::WriteDescriptorSet::builder()
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .dst_array_element(0)
            .build()];

        let layout_binding = [vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .binding(0)
            .build()];

        let input_buffer_layout = BufferLayout::new().add_float_vec3(0);

        let mut builder = ShaderBuilder::builder(
            application,
            "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\grid_vert.spv",
            "E:\\programming\\Anasazi\\anasaizi-editor\\assets\\grid_frag.spv",
            3,
        );
        builder
            .with_input_buffer_layout(input_buffer_layout)
            .with_write_descriptor_layout(&layout_binding)
            .with_descriptor_pool(&[vk::DescriptorType::UNIFORM_BUFFER])
            .with_write_descriptor_sets(descriptor_write_sets);

        let build: ShaderSet<U> = builder.build();

        let pipeline = Pipeline::create(
            &application.device,
            self.swapchain.extent,
            &self.render_pass,
            &build,
        );

        RenderObject {
            pipeline,
            mesh: Mesh::new(grid_vertex_buffer, grid_index_buffer),
            shader: build,
        }
    }
}
