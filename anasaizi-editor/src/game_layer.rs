use anasaizi_core::{
    debug::stop_profiler,
    engine,
    engine::{Layer, RenderContext, VulkanApplication},
};
use std::{
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Instant,
};
use winit::{
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

pub struct GameLayer {
    input_receiver: Receiver<engine::Event>,
    input_sender: Sender<engine::Event>,
    last_frame: Instant,
    delta_time: u128,
    input: Vec<engine::Event>,
}

impl GameLayer {
    pub fn new() -> GameLayer {
        let (tx, rx) = channel();

        GameLayer {
            input_sender: tx,
            input_receiver: rx,
            last_frame: Instant::now(),
            delta_time: 0,
            input: vec![],
        }
    }
}

impl GameLayer {
    pub fn tick(&mut self, event_loop: &mut EventLoop<()>) -> bool {
        let now = Instant::now();
        self.delta_time = (now - self.last_frame).as_millis();
        let mut run = true;

        event_loop.run_return(|winit_event, _, control_flow| {
            let winit_event = winit::event::Event::to_static(winit_event).unwrap();
            let ark = Arc::from(winit_event.clone());

            self.input_sender.send(engine::Event::Raw(ark));

            match winit_event {
                winit::event::Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        stop_profiler();
                        self.input_sender.send(engine::Event::Shutdown);
                        run = false;
                    }
                    WindowEvent::CursorMoved { position, modifiers, .. } => {
                        self.input_sender.send(engine::Event::MouseMove(position,modifiers));
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        if let MouseScrollDelta::LineDelta(x, y) = delta {
                            self.input_sender.send(engine::Event::MouseScroll(x, y));
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        self.input_sender.send(engine::Event::Keyboard(input));
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        self.input_sender
                            .send(engine::Event::MouseInput(state, button));
                    }
                    _ => {}
                },
                _ => (),
            };
            *control_flow = ControlFlow::Exit;
        });

        run
    }

    pub fn before_frame(&mut self) {
        while let Ok(event) = self.input_receiver.try_recv() {
            self.input.push(event);
        }
    }

    pub fn run_layers<T: Layer>(
        &mut self,
        layers: &mut Vec<T>,
        render_context: &RenderContext,
        application: &VulkanApplication,
    ) {
        for layer in layers.iter_mut() {
            for event in self.input.iter() {
                layer.on_event(&event);
            }
        }

        // Then handle frame start.
        for layer in layers.iter_mut() {
            layer.start_frame();
        }

        for layer in layers.iter_mut() {
            layer.on_update(self.delta_time, render_context, application);
        }

        for layer in layers.iter_mut() {
            layer.end_frame();
        }
    }

    pub fn after_frame(&mut self) {
        // First handle events.
        self.input.clear();
    }
}
