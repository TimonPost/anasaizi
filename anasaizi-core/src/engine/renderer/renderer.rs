use crate::{
    engine::{
        camera::{Camera, CameraMovement},
        image::Texture,
        Event, VulkanApplication,
    },
    model::Mesh,
    profile_fn,
    vulkan::{
        structures::SyncObjects, CommandBuffers, CommandPool, FrameBuffers, Queue,
        RenderPass, ShaderSet, SwapChain, UniformBufferObjectTemplate,
    },
};
use ultraviolet::projection::orthographic_vk;

use anasaizi_profile::profile;

use crate::{
    math::PosOnlyVertex,
    model::{square_indices, square_vertices},
    reexports::imgui::{DrawCmd, DrawCmdParams, DrawData, TextureId},
    vulkan::{
        IndexBuffer, Instance, LogicalDevice, Pipeline, ShaderBuilder, UniformBufferObject,
        VertexBuffer,
    },
};
use ash::{version::DeviceV1_0, vk};
use std::{ptr, time::Instant};
use winit::event::{ElementState, VirtualKeyCode};
use crate::engine::BufferLayout;

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

pub struct VulkanRenderer<U: UniformBufferObjectTemplate> {
    pub swapchain: SwapChain,
    pub render_pass: RenderPass,
    pub graphics_queue: Queue,
    present_queue: Queue,
    pub command_pool: CommandPool,

    frame_buffers: FrameBuffers,
    command_buffers: CommandBuffers,

    pub pipelines: Vec<Pipeline<U>>,
    pub ui_pipeline: Option<Pipeline<U>>,

    pub texture_sampler: Option<vk::Sampler>,

    sync_object: SyncObjects,

    pub current_frame: usize,
    pub last_frame: Instant,
    pub delta_time: f32,
    pub start_time: Instant,

    pub camera: Camera,
    pub last_y: f64,
    pub last_x: f64,

    ui_mesh: Option<Mesh>,
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

        VulkanRenderer {
            swapchain,
            render_pass,

            command_pool,

            graphics_queue,
            present_queue,

            frame_buffers,
            command_buffers,
            pipelines: vec![],
            ui_pipeline: None,

            texture_sampler: Some(texture_sampler),

            sync_object,

            camera,

            delta_time: 0.0,

            last_frame: Instant::now(),
            start_time: Instant::now(),
            current_frame: 0,

            last_x: 400.0,
            last_y: 300.0,

            ui_mesh: None,
        }
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
        meshes: Vec<Mesh>,
    ) {
        let mut pipeline = Pipeline::create(
            &application.device,
            self.swapchain.extent,
            &self.render_pass,
            shader,
        );
        pipeline.meshes = meshes;

        self.pipelines.push(pipeline);
    }

    pub fn initialize_resources() {}

    pub fn start_frame(&mut self) {
        let current_frame = Instant::now();

        self.delta_time = (current_frame - self.last_frame).as_millis() as f32;
        self.last_frame = current_frame;
    }

    #[profile(VulkanRenderer)]
    pub fn draw(&mut self, application: &VulkanApplication, draw_data: &DrawData) {
        let device = &application.device;

        let wait_fences = [self.sync_object.inflight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            profile_fn!("Acquire Next Image...", {
                unsafe {
                    device
                        .reset_command_pool(*self.command_pool, vk::CommandPoolResetFlags::empty());
                }

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
                device,
                &self.render_pass,
                self.swapchain.extent,
                &self.frame_buffers,
                self.current_frame,
            );

            self.render_meshes(&application);

            unsafe {
                device
                    .cmd_next_subpass(self.command_buffers.current(), vk::SubpassContents::INLINE);
            };

            self.render_ui(&application, draw_data, self.current_frame);

            self.command_buffers.end_session(device);
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
                            self.recreate_swapchain(application);
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

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn render_meshes(&self, application: &VulkanApplication) {
        unsafe {
            let viewports = [vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }];

            application
                .device
                .cmd_set_viewport(self.command_buffers.current(), 0, &viewports);

            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            }];

            application
                .device
                .cmd_set_scissor(self.command_buffers.current(), 0, &scissors);
        };

        for pipeline in self.pipelines.iter() {
            self.command_buffers
                .bind_pipeline(&application.device, pipeline);

            for mesh in pipeline.meshes.iter() {
                self.command_buffers
                    .render_mesh(&application.device, pipeline, &mesh);
            }
        }
    }

    pub fn render_ui(
        &mut self,
        application: &VulkanApplication,
        draw_data: &DrawData,
        image_index: usize,
    ) {
        if let Some(ui_pipeline) = &self.ui_pipeline {
            self.command_buffers
                .bind_pipeline(&application.device, ui_pipeline);
            let current_command_buffer = self.command_buffers.current();

            if let None = self.ui_mesh {
                self.ui_mesh = Some(Mesh::from_draw_data(
                    &application.instance,
                    &application.device,
                    &self.graphics_queue,
                    &self.command_pool,
                    &draw_data,
                ));
            } else {
                let mut mesh = self.ui_mesh.as_mut().unwrap();
                mesh.update_from_draw_data(
                    &application.instance,
                    &application.device,
                    &self.graphics_queue,
                    &self.command_pool,
                    draw_data,
                );
            }
            let mesh = self.ui_mesh.as_ref().unwrap();

            let device = &application.device;

            let framebuffer_width = draw_data.framebuffer_scale[0] * draw_data.display_size[0];
            let framebuffer_height = draw_data.framebuffer_scale[1] * draw_data.display_size[1];
            let viewports = [vk::Viewport {
                width: framebuffer_width,
                height: framebuffer_height,
                max_depth: 1.0,
                ..Default::default()
            }];

            unsafe { device.cmd_set_viewport(current_command_buffer, 0, &viewports) };

            // Ortho projection
            let projection = orthographic_vk(
                0.0,
                draw_data.display_size[0],
                0.0,
                -draw_data.display_size[1],
                -1.0,
                1.0,
            );

            unsafe {
                let push = any_as_u8_slice(&projection);
                device.cmd_push_constants(
                    current_command_buffer,
                    ui_pipeline.layout(),
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push,
                )
            };

            unsafe {
                device.cmd_bind_index_buffer(
                    current_command_buffer,
                    **mesh.index_buffer(),
                    0,
                    vk::IndexType::UINT32,
                )
            };

            unsafe {
                device.cmd_bind_vertex_buffers(
                    current_command_buffer,
                    0,
                    &[**mesh.vertex_buffer()],
                    &[0],
                )
            };

            let mut index_offset = 0;
            let mut vertex_offset = 0;

            let clip_offset = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;

            for draw_list in draw_data.draw_lists() {
                for command in draw_list.commands() {
                    if let DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                            },
                    } = command
                    {
                        unsafe {
                            let clip_x = (clip_rect[0] - clip_offset[0]) * clip_scale[0];
                            let clip_y = (clip_rect[1] - clip_offset[1]) * clip_scale[1];
                            let clip_w = (clip_rect[2] - clip_offset[0]) * clip_scale[0] - clip_x;
                            let clip_h = (clip_rect[3] - clip_offset[1]) * clip_scale[1] - clip_y;

                            let scissors = [vk::Rect2D {
                                offset: vk::Offset2D {
                                    x: clip_x as _,
                                    y: clip_y as _,
                                },
                                extent: vk::Extent2D {
                                    width: clip_w as _,
                                    height: clip_h as _,
                                },
                            }];
                            device.cmd_set_scissor(current_command_buffer, 0, &scissors);
                            let sets = [*ui_pipeline.shader.descriptor_sets[image_index]];
                            device.cmd_bind_descriptor_sets(
                                current_command_buffer,
                                vk::PipelineBindPoint::GRAPHICS,
                                ui_pipeline.layout(),
                                0,
                                &sets,
                                &[],
                            );

                            device.cmd_draw_indexed(
                                current_command_buffer,
                                count as _,
                                1,
                                index_offset + idx_offset as u32,
                                vertex_offset + vtx_offset as i32,
                                0,
                            )
                        }
                    }
                }
            }
        }
    }

    pub fn end_frame() {}

    pub fn recreate_swapchain(&mut self, application: &VulkanApplication) {
        unsafe {
            application
                .device
                .device_wait_idle()
                .expect("Failed to wait device idle!");

            self.destroy_swapchain(&application.device);
        };

        let surface_data = application.window.surface_data();

        self.swapchain = SwapChain::new(&application.instance, &application.device, surface_data);
        self.render_pass = RenderPass::create(
            &application.instance,
            &application.device,
            self.swapchain.image_format,
        );
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
            },
            Event::MouseScroll(_xoffset, yoffset) => {
                self.camera.process_mouse_scroll(yoffset);
            }
            _ => {}
        }
    }

    pub fn grid_mesh(&self, application: &VulkanApplication) -> (ShaderSet<U>, Mesh) {
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

        let descriptor_write_sets = vec![vk::WriteDescriptorSet::builder()
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
            "assets\\shaders\\build\\grid_vert.spv",
            "assets\\shaders\\build\\grid_frag.spv",
            self.swapchain.images.len(),
        );
        builder
            .with_input_buffer_layout(input_buffer_layout)
            .with_write_descriptor_layout(&layout_binding)
            .with_descriptor_pool(&[vk::DescriptorType::UNIFORM_BUFFER])
            .with_write_descriptor_sets(descriptor_write_sets);

        let build: ShaderSet<U> = builder.build();

        (build, Mesh::new(grid_vertex_buffer, grid_index_buffer))
    }
}

/// Return a `&[u8]` for any sized object passed in.
pub(crate) unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}
