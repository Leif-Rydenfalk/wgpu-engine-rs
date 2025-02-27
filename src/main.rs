#![feature(portable_simd)]
use std::thread;
use std::time::Instant;

use std::simd::num::*;
use std::simd::*;
use std::simd::cmp::*;

use crate::wgpu_ctx::InstanceData;
use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};

use std::sync::mpsc::{channel, Sender};

use rand::Rng;

mod input;
pub use input::*;
mod app;
mod wgpu_ctx;
mod camera;
pub use camera::*;

fn sim(sender: Sender<Vec<InstanceData>>) {
    const FRAMES: u32 = 60;
    const COUNT: usize = 1_000; 
    const GRID_SIZE: usize = COUNT.isqrt();
    const SPACING: f32 = 2.0; // Space between particles

    // Define boundaries
    const BOUNDS_X: f32 = 1_000.0; // Maximum x distance from center
    const BOUNDS_Y: f32 = 1_000.0; // Maximum y distance from center
    const SIMD_LEVEL: usize = 32;
    let bounds_x_max = f32x32::splat(BOUNDS_X);
    let bounds_x_min = f32x32::splat(-BOUNDS_X);
    let bounds_y_max = f32x32::splat(BOUNDS_Y);
    let bounds_y_min = f32x32::splat(-BOUNDS_Y);
    
    let mut x = vec![f32x32::splat(0.0f32); COUNT / SIMD_LEVEL + 1];
    let mut y = vec![f32x32::splat(0.0f32); COUNT / SIMD_LEVEL + 1];
    let mut x_vel = vec![f32x32::splat(0.0f32); COUNT / SIMD_LEVEL + 1];
    let mut y_vel = vec![f32x32::splat(0.0f32); COUNT / SIMD_LEVEL + 1];
    
    let mut rng = rand::rng(); // Fixed random number generator initialization
    
    // Initialize grid positions
    for i in 0..COUNT / SIMD_LEVEL {
        let mut x_values = [0.0f32; SIMD_LEVEL];
        let mut y_values = [0.0f32; SIMD_LEVEL];
        
        for j in 0..SIMD_LEVEL {
            let index = i * SIMD_LEVEL + j;
            if index < COUNT {
                let row = index / GRID_SIZE;
                let col = index % GRID_SIZE;
                
                // Center the grid and offset each particle
                x_values[j] = (col as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
                y_values[j] = (row as f32 - GRID_SIZE as f32 / 2.0) * SPACING;
            }
        }
        
        x[i] = f32x32::from_array(x_values);
        y[i] = f32x32::from_array(y_values);
    }

    // Initialize velocities
    for i in 0..COUNT / SIMD_LEVEL {
        let mut x_values = [0.0f32; SIMD_LEVEL];
        let mut y_values = [0.0f32; SIMD_LEVEL];
        
        for j in 0..SIMD_LEVEL {
            // Generate random velocities in range -1.0 to 1.0
            x_values[j] = rng.random_range(-1.0..1.0) * 1.0;
            y_values[j] = rng.random_range(-1.0..1.0) * 1.0;
        }
        
        x_vel[i] = f32x32::from_array(x_values);
        y_vel[i] = f32x32::from_array(y_values);
    }
    
    let gravity = f32x32::splat(-0.0f32);
    let dt = f32x32::splat(0.1f32);
   
    for frame in 0.. {
        let frame_start = Instant::now();

        // Update physics
        for i in 0..COUNT / SIMD_LEVEL {
            y_vel[i] += gravity * dt;
            x[i] += x_vel[i] * dt;
            y[i] += y_vel[i] * dt;

            // Apply boundary constraints with velocity reflection
            let x_gt_max = x[i].simd_gt(bounds_x_max);
            let x_lt_min = x[i].simd_lt(bounds_x_min);
            let y_gt_max = y[i].simd_gt(bounds_y_max);  
            let y_lt_min = y[i].simd_lt(bounds_y_min);
            
            // Clamp positions to bounds
            x[i] = x[i].simd_min(bounds_x_max).simd_max(bounds_x_min);
            y[i] = y[i].simd_min(bounds_y_max).simd_max(bounds_y_min);
            
            // Reverse velocities at boundaries (with some energy loss)
            let bounce_factor = f32x32::splat(-0.8); // 20% energy loss on bounce
            x_vel[i] = x_vel[i] * (x_gt_max.select(bounce_factor, f32x32::splat(1.0))) 
                                * (x_lt_min.select(bounce_factor, f32x32::splat(1.0)));
            y_vel[i] = y_vel[i] * (y_gt_max.select(bounce_factor, f32x32::splat(1.0)))
                                * (y_lt_min.select(bounce_factor, f32x32::splat(1.0)));     
        }

        // Convert SIMD data to instance data
        let mut instances = Vec::with_capacity(COUNT);
        for i in 0..COUNT / SIMD_LEVEL {
            let x_array = x[i].as_array();
            let y_array = y[i].as_array();
            
            for j in 0..SIMD_LEVEL {
                let index = i * SIMD_LEVEL + j;
                if index < COUNT {
                    instances.push(InstanceData {
                        position: [x_array[j], y_array[j]],
                    });
                }
            }
        }

        // Send updated instances to renderer
        if sender.send(instances).is_err() {
            break; // Exit if receiver is dropped
        }

        if frame % FRAMES == 0 {
            let elapsed = frame_start.elapsed();
            println!("{:#?} {:#?}fps", elapsed, 1.0 / elapsed.as_secs_f32());
        }

        thread::sleep(std::time::Duration::from_millis(16).saturating_sub(frame_start.elapsed())); // ~60 FPS
    }
}

fn main()  {
    const STACK_SIZE: usize = 2 * 128 * 1_000_000;
    
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



// mod bench;

// pub use bench::*;

// fn main() {
//     const STACK_SIZE: usize = 128 * 1_000_000;

//     // Spawn simulation thread
//     let sim_thread = thread::Builder::new()
//         .stack_size(STACK_SIZE)
//         .spawn(move || simd_bench())
//         .unwrap();

//     sim_thread.join().unwrap();
// }
