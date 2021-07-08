use anasaizi_core::vulkan::{structures::{SyncObjects, ValidationInfo}, Application, CommandBuffers, CommandPool, Extensions, FrameBuffers, Instance, LogicalDevice, Pipeline, Queue, RenderPass, Shader, Shaders, SwapChain, Version, Window, VertexBuffer};
use ash::vk;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use anasaizi_core::{profile_fn, WINDOW_WIDTH, WINDOW_HEIGHT};
use anasaizi_profile::profile;

use anasaizi_core::debug::{start_profiler, stop_profiler};
use ash::{
    extensions::{ext::DebugUtils, khr},
    version::DeviceV1_0,
};
use std::ptr;
use anasaizi_core::model::{triangle_vertices, Mesh, square_vertices};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub const VALIDATION: ValidationInfo = ValidationInfo {
    is_enable: true,
    required_validation_layers: ["VK_LAYER_KHRONOS_validation"],
};

pub static FRAGMENT_SHADER: &str = "frag.spv";
pub static VERTEX_SHADER: &str = "vert.spv";

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

pub struct VulkanApp {
    device: LogicalDevice,
    application: Application,
    swapchain: SwapChain,
    render_pass: RenderPass,
    pipeline: Pipeline,
    shaders: Shaders,
    frame_buffers: FrameBuffers,
    command_pool: CommandPool,
    buffers: CommandBuffers,
    graphics_queue: Queue,
    present_queue: Queue,
    instance: Instance,
    window: Window,
    sync_object: SyncObjects,

    current_frame: usize,
    pub square_mesh: Mesh
}

impl VulkanApp {
    pub fn new(event_loop: &EventLoop<()>) -> VulkanApp {
        let instance_extensions = Extensions::new(vec![
            khr::Surface::name().to_str().unwrap().to_string(),
            khr::Win32Surface::name().to_str().unwrap().to_string(),
            DebugUtils::name().to_str().unwrap().to_string(),
        ]);

        let device_extensions =
            Extensions::new(vec![khr::Swapchain::name().to_str().unwrap().to_string()]);

        let application = Application::new(
            "Engine",
            "Vulkan Engine",
            Version::new(0, 0, 1),
            Version::new(1, 2, 0),
            Version::new(1, 2, 0),
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        );

        let instance = Instance::new(VALIDATION, instance_extensions, &application);

        let window = Window::new("Engine", WINDOW_WIDTH, WINDOW_HEIGHT, &instance, event_loop);

        let device = LogicalDevice::new(&instance, device_extensions, window.surface_data());

        let graphics_queue = Queue::create(
            &device,
            device.queue_family_indices().graphics_family.unwrap(),
        );
        let present_queue = Queue::create(
            &device,
            device.queue_family_indices().present_family.unwrap(),
        );

        let swapchain = SwapChain::new(&instance, &device, window.surface_data());

        let render_pass = RenderPass::create(&device, swapchain.image_format);

        let mut shaders = Shaders::new();
        shaders.add_shader(VERTEX_SHADER, Shader::new(&device, VERTEX_SHADER));
        shaders.add_shader(FRAGMENT_SHADER, Shader::new(&device, FRAGMENT_SHADER));

        let pipeline = Pipeline::create(
            &device,
            swapchain.extent,
            &render_pass,
            shaders.shader(VERTEX_SHADER),
            shaders.shader(FRAGMENT_SHADER),
        );

        let frame_buffers = FrameBuffers::create(
            &device,
            &render_pass,
            &swapchain.image_views,
            &swapchain.extent,
        );

        let command_pool = CommandPool::create(&device);

        // let triangle_vertices = triangle_vertices().to_vec();
        // let vertex_buffer = VertexBuffer::create(&instance, &device, &triangle_vertices, &graphics_queue, &command_pool);
        // let triangle_mesh = Mesh::new(vertex_buffer, triangle_vertices);

        let square_vertices = square_vertices().to_vec();
        let square_vertex_buffer = VertexBuffer::create(&instance, &device, &square_vertices, &graphics_queue, &command_pool);
        let square_mesh = Mesh::new(square_vertex_buffer, square_vertices);

        let buffers = CommandBuffers::create(
            &device,
            &command_pool,
            &pipeline,
            &frame_buffers,
            &render_pass,
            swapchain.extent,
            &square_mesh
        );

        let sync_object = create_sync_objects(device.logical_device());

        println!("{:?}", application);
        println!("{:?}", instance);
        println!("{:?}", device);

        start_profiler();

        VulkanApp {
            window,
            application,
            instance,
            device,

            swapchain,
            render_pass,
            pipeline,

            shaders,

            frame_buffers,
            command_pool,
            buffers,
            graphics_queue,
            present_queue,

            sync_object,
            current_frame: 0,
            square_mesh
        }
    }

    #[profile(Sandbox)]
    fn draw_frame(&mut self) {
        let wait_fences = [self.sync_object.inflight_fences[self.current_frame]];

        let (image_index, _is_sub_optimal) = unsafe {
            profile_fn!("Acquire Next Image...", {
                self.device
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
        profile_fn!("Wrapper Queues...", {
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
                p_command_buffers: &self.buffers.get(image_index as usize),
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
            }];

            unsafe {
                profile_fn!("Submitting Queues...", {
                    self.device
                        .reset_fences(&wait_fences)
                        .expect("Failed to reset Fence!");

                    self.device
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

    pub fn main_loop(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;

                    stop_profiler();
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            *control_flow = ControlFlow::Exit
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                self.draw_frame();
            }
            Event::LoopDestroyed => {
                unsafe {
                    self.device
                        .device_wait_idle()
                        .expect("Failed to wait device idle!")
                };
            }
            _ => (),
        })
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        // unsafe {
        //     // self.device.destroy_shader_module(vert_shader_module, None);
        //     // self.device.destroy_shader_module(frag_shader_module, None);
        //
        //     for &imageview in self.image_views.iter() {
        //         self.device.destroy_image_view(imageview, None);
        //     }
        //
        //     self.swapchain_data
        //         .swapchain_loader
        //         .destroy_swapchain(self.swapchain_data.swapchain, None);
        //     self.device.destroy_device(None);
        //     self.surface_data
        //         .surface_loader
        //         .destroy_surface(self.surface_data.surface, None);
        //
        //     if VALIDATION.is_enable {
        //         self.debug_utils_loader
        //             .destroy_debug_utils_messenger(self.debug_merssager, None);
        //     }
        //     self.instance.destroy_instance(None);
        // }
    }
}
