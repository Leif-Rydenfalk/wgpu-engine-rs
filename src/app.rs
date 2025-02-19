use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::wgpu_ctx::InstanceData;

use std::sync::mpsc::Receiver;

use crate::wgpu_ctx::WgpuCtx;

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub wgpu_ctx: Option<WgpuCtx<'window>>,
    pub instance_receiver: Receiver<Vec<InstanceData>>,
    pub mouse_position: Option<[f32; 2]>,
}

impl App<'_> {
    pub fn new(instance_receiver: Receiver<Vec<InstanceData>>) -> Self {
        Self {
            instance_receiver,
            window: None,
            mouse_position: None,
            wgpu_ctx: None
        }
    }
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attr = Window::default_attributes().with_title("wgpu simulation");
            let window = Arc::new(
                event_loop
                    .create_window(win_attr)
                    .expect("create window err."),
            );
            self.window = Some(window.clone());
            let wgpu_ctx = WgpuCtx::new(window.clone());
            self.wgpu_ctx = Some(wgpu_ctx);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let (Some(wgpu_ctx), Some(window)) =
                    (self.wgpu_ctx.as_mut(), self.window.as_ref())
                {
                    wgpu_ctx.resize((new_size.width, new_size.height));
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    // Check for new instance data
                    if let Ok(new_instances) = self.instance_receiver.try_recv() {
                        wgpu_ctx.update_instances(&new_instances);
                    }
                    wgpu_ctx.draw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let (Some(wgpu_ctx), Some(mouse_position)) = (self.wgpu_ctx.as_mut(), self.mouse_position) {
                    let scroll_amount = match delta {
                        MouseScrollDelta::LineDelta(_, y) => y,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    wgpu_ctx.camera.handle_scroll(scroll_amount, mouse_position);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    match state {
                        ElementState::Pressed => {
                            if let Some(position) = self.mouse_position {
                                wgpu_ctx.camera.handle_mouse_press(button, position);
                            }
                        }
                        ElementState::Released => {
                            wgpu_ctx.camera.handle_mouse_release(button);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some([position.x as f32, position.y as f32]);
                if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                    wgpu_ctx.camera.handle_mouse_move([position.x as f32, position.y as f32]);
                }
            }
            _ => (),
        }
    }
}
