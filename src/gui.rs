use bytemuck::{Pod, Zeroable};
use imgui::*;
use imgui_wgpu::{Renderer, RendererConfig, Texture, TextureConfig};
use imgui_winit_support::WinitPlatform;
use pollster::block_on;
use std::{sync::Arc, time::Instant};
use wgpu::{include_wgsl, util::DeviceExt, Extent3d};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::Window,
};

