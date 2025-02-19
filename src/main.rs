use std::thread;
use std::time::Instant;

use wide::{f32x4, f32x8};

use crate::wgpu_ctx::InstanceData;
use crate::app::App;
use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

use std::sync::mpsc::{channel, Receiver, Sender};

use rand::Rng;

mod app;
mod wgpu_ctx;
mod camera;
pub use camera::*;

fn sim(sender: Sender<Vec<InstanceData>>) {
    const FRAMES: u32 = 60;
    const COUNT: usize = 10000; 
    const GRID_SIZE: usize = 100; // sqrt(COUNT)
    const SPACING: f32 = 0.2; // Space between particles
    
    let mut x = [f32x4::new([0.0f32; 4]); COUNT / 4 + 1];
    let mut y = [f32x4::new([0.0f32; 4]); COUNT / 4 + 1];
    let mut x_vel = [f32x4::new([0.0f32; 4]); COUNT / 4 + 1];
    let mut y_vel = [f32x4::new([0.0f32; 4]); COUNT / 4 + 1];

    let mut rng = rand::rng();
    
    // Initialize grid positions
    for i in 0..COUNT/4 {
        let mut x_values = [0.0f32; 4];
        let mut y_values = [0.0f32; 4];
        
        for j in 0..4 {
            let index = i * 4 + j;
            if index < COUNT {
                let row = index / GRID_SIZE;
                let col = index % GRID_SIZE;
                
                // Center the grid and offset each particle
                x_values[j] = (col as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
                y_values[j] = (row as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
            }
        }
        
        x[i] = f32x4::new(x_values);
        y[i] = f32x4::new(y_values);
    }

    // Initialize velocities
    for i in 0..COUNT/4 {
        let mut x_values = [0.0f32; 4];
        let mut y_values = [0.0f32; 4];
        
        for j in 0..4 {
            // Center the grid and offset each particle
            x_values[j] = rng.random_range(-0.1..0.1);
            y_values[j] = rng.random_range(-0.1..0.1);
        }
        
        x_vel[i] = f32x4::new(x_values);
        y_vel[i] = f32x4::new(y_values);
    }
    
    let gravity: f32x4 = f32x4::new([-0.01f32; 4]);
    let dt = f32x4::new([0.1f32; 4]);
    
    let mut frame: u32 = 0;
    loop {
        let frame_start = Instant::now();
        frame += 1;

        // Update physics
        for i in 0..COUNT/4 {
            y_vel[i] += gravity * dt;
            x[i] += x_vel[i] * dt;
            y[i] += y_vel[i] * dt;
        }

        // Convert SIMD data to instance data
        let mut instances = Vec::with_capacity(COUNT);
        instances.extend((0..COUNT/4).flat_map(|i| {
            let x_array = x[i].to_array();
            let y_array = y[i].to_array();
            (0..4).map(move |j| InstanceData {
                position: [x_array[j], y_array[j]],
            })
        }));

        // Send updated instances to renderer
        if sender.send(instances).is_err() {
            break; // Exit if receiver is dropped
        }

        if frame % 60 == 0 {
            let elapsed = frame_start.elapsed();
            println!("{:#?} {:#?}fps", elapsed, 1.0 / elapsed.as_secs_f32());
        }

        thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
    }
}

fn main()  {
    const STACK_SIZE: usize = 128 * 1_000_000;
    
    // Create channel for instance data
    let (sender, receiver) = channel();

    // Spawn simulation thread
    let sim_thread = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || sim(sender))
        .unwrap();

    // Create event loop with receiver
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(receiver);
    event_loop.run_app(&mut app).unwrap();
    sim_thread.join().unwrap();
}
