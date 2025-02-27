use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::wgpu_ctx::InstanceData;
use crate::wgpu_ctx::WgpuCtx;

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    pub renderer: imgui_wgpu::Renderer,
    pub last_frame: std::time::Instant,
    pub last_cursor: Option<imgui::MouseCursor>,
}

pub struct App<'window> {
    pub window: Option<Arc<Window>>,
    pub wgpu_ctx: Option<WgpuCtx<'window>>,
    pub instance_receiver: Receiver<Vec<InstanceData>>,
    pub mouse_position: Option<[f32; 2]>,
    pub imgui: Option<ImguiState>,
    pub input: input_actions::System,
}

impl<'window> App<'window> {
    fn setup_imgui(&mut self) {
        if let (Some(window), Some(wgpu_ctx)) = (self.window.as_ref(), self.wgpu_ctx.as_ref()) {
            let mut context = imgui::Context::create();
            let mut platform = imgui_winit_support::WinitPlatform::new(&mut context);

            // Attach window
            platform.attach_window(
                context.io_mut(),
                window,
                imgui_winit_support::HiDpiMode::Default,
            );

            // Disable .ini file
            context.set_ini_filename(None);

            // Set up font
            let hidpi_factor = window.scale_factor();
            let font_size = (13.0 * hidpi_factor) as f32;
            context.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

            context
                .fonts()
                .add_font(&[imgui::FontSource::DefaultFontData {
                    config: Some(imgui::FontConfig {
                        oversample_h: 1,
                        pixel_snap_h: true,
                        size_pixels: font_size,
                        ..Default::default()
                    }),
                }]);

            // Create renderer
            let renderer_config = imgui_wgpu::RendererConfig {
                texture_format: wgpu_ctx.surface_config.format,
                ..Default::default()
            };

            let renderer = imgui_wgpu::Renderer::new(
                &mut context,
                &wgpu_ctx.device,
                &wgpu_ctx.queue,
                renderer_config,
            );

            self.imgui = Some(ImguiState {
                context,
                platform,
                renderer,
                last_frame: std::time::Instant::now(),
                last_cursor: None,
            });
        }
    }

    fn render(&mut self) {
        if let (Some(imgui_state), Some(wgpu_ctx), Some(window)) = (
            &mut self.imgui,
            self.wgpu_ctx.as_mut(),
            self.window.as_ref(),
        ) {
            // Update frame timing and prepare ImGui
            let now = std::time::Instant::now();
            {
                let io = imgui_state.context.io_mut();
                io.update_delta_time(now - imgui_state.last_frame);
            }
            imgui_state.last_frame = now;
            imgui_state
                .platform
                .prepare_frame(imgui_state.context.io_mut(), window)
                .expect("Failed to prepare frame");

            // Build your ImGui UI
            let ui = imgui_state.context.frame();
            ui.window("Debug")
                .size([300.0, 200.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    ui.text("Hello from ImGui!");
                    ui.text(format!("FPS: {:.1}", ui.io().framerate));
                });

            // Acquire the swap chain texture only once
            let output = wgpu_ctx
                .surface
                .get_current_texture()
                .expect("Failed to get current texture");
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            // Create one command encoder for both passes
            let mut encoder =
                wgpu_ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Main Command Encoder"),
                    });

            if let Ok(new_instances) = self.instance_receiver.try_recv() {
                wgpu_ctx.update_instances(&new_instances);
            }

            let view_matrix = wgpu_ctx.camera.get_view_matrix();
            wgpu_ctx.queue.write_buffer(
                &wgpu_ctx.uniform_buffer,
                0,
                bytemuck::cast_slice(&view_matrix),
            );

            // --- Scene Render Pass ---
            {
                let mut scene_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Scene Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Render your scene
                scene_pass.set_pipeline(&wgpu_ctx.render_pipeline);
                scene_pass.set_vertex_buffer(0, wgpu_ctx.vertex_buffer.slice(..));
                scene_pass.set_vertex_buffer(1, wgpu_ctx.instance_buffer.slice(..));
                scene_pass.set_bind_group(0, &wgpu_ctx.uniform_bind_group, &[]);
                scene_pass.draw(0..3, 0..wgpu_ctx.num_instances);
            } // End scene pass

            // --- ImGui Render Pass ---
            {
                // Use Load so that the scene remains intact
                let mut imgui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("ImGui Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // Render ImGui draw data on top of the scene
                let draw_data = imgui_state.context.render();
                imgui_state
                    .renderer
                    .render(
                        draw_data,
                        &wgpu_ctx.queue,
                        &wgpu_ctx.device,
                        &mut imgui_pass,
                    )
                    .expect("Failed to render ImGui");
            } // End ImGui pass

            // Submit all commands and present
            wgpu_ctx.queue.submit(Some(encoder.finish()));
            output.present();
        }
    }
}

impl App<'_> {
    pub fn new(instance_receiver: Receiver<Vec<InstanceData>>) -> Self {
        Self {
            instance_receiver,
            window: None,
            mouse_position: None,
            wgpu_ctx: None,
            imgui: None,
            input: input_actions::System::new(),
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

            // Setup ImGui after initializing WGPU
            self.setup_imgui();
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
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Ok(input_event) = input_actions::winit::parse_winit_event(&event) {
            self.input.send_event(input_event.0, input_event.1);
        }

        // First let ImGui process the event
        if let Some(imgui_state) = &mut self.imgui {
            imgui_state.platform.handle_event::<WindowEvent>(
                imgui_state.context.io_mut(),
                self.window.as_ref().unwrap(),
                &winit::event::Event::WindowEvent {
                    window_id,
                    event: event.clone(),
                },
            );
        }

        // Check if ImGui wants to capture mouse input
        let imgui_captures_mouse = self.imgui.as_ref().map_or(false, |imgui_state| {
            imgui_state.context.io().want_capture_mouse
        });

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
                self.render();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // Only handle scroll if ImGui is not capturing the mouse
                if !imgui_captures_mouse {
                    if let (Some(wgpu_ctx), Some(mouse_position)) =
                        (self.wgpu_ctx.as_mut(), self.mouse_position)
                    {
                        let scroll_amount = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                        };
                        wgpu_ctx.camera.handle_scroll(scroll_amount, mouse_position);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if !imgui_captures_mouse {
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
            }
            WindowEvent::CursorMoved { position, .. } => {
                // Update the cursor position regardless of ImGui capture
                self.mouse_position = Some([position.x as f32, position.y as f32]);
                // Only update camera movement if ImGui is not capturing mouse events
                if !imgui_captures_mouse {
                    if let Some(wgpu_ctx) = self.wgpu_ctx.as_mut() {
                        wgpu_ctx
                            .camera
                            .handle_mouse_move([position.x as f32, position.y as f32]);
                    }
                }
            }
            _ => (),
        }
    }
}
