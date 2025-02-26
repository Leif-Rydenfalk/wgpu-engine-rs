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

use std::collections::HashMap;

// Cell size should be slightly larger than the maximum interaction distance
const CELL_SIZE: f32 = 10.0; 
const COLLISION_RADIUS: f32 = 0.2; // Collision distance between particles

// Hash grid for collision detection
struct HashGrid {
    cells: HashMap<(i32, i32), Vec<usize>>,
    cell_size: f32,
}

impl HashGrid {
    fn new(cell_size: f32) -> Self {
        HashGrid {
            cells: HashMap::new(),
            cell_size,
        }
    }

    // Insert particle with global index into the grid
    fn insert(&mut self, x: f32, y: f32, index: usize) {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        self.cells.entry((cell_x, cell_y)).or_insert_with(Vec::new).push(index);
    }

    // Get potential collision candidates for a particle
    fn get_potential_collisions(&self, x: f32, y: f32) -> Vec<usize> {
        let cell_x = (x / self.cell_size).floor() as i32;
        let cell_y = (y / self.cell_size).floor() as i32;
        
        let mut candidates = Vec::new();
        
        // Check the cell containing the particle and all 8 surrounding cells
        for dy in -1..=1 {
            for dx in -1..=1 {
                if let Some(indices) = self.cells.get(&(cell_x + dx, cell_y + dy)) {
                    candidates.extend(indices);
                }
            }
        }
        
        candidates
    }
}

// Function to check and resolve collisions using the hash grid
fn resolve_collisions(
    x: &mut [f32x32], 
    y: &mut [f32x32], 
    x_vel: &mut [f32x32], 
    y_vel: &mut [f32x32], 
    count: usize,
    simd_level: usize
) {
    let mut grid = HashGrid::new(CELL_SIZE);
    
    // First, populate the grid with all particles
    for i in 0..count / simd_level {
        let x_array = x[i].as_array();
        let y_array = y[i].as_array();
        
        for j in 0..simd_level {
            let index = i * simd_level + j;
            if index < count {
                grid.insert(x_array[j], y_array[j], index);
            }
        }
    }
    
    // Define collision response parameters
    let repulsion_strength = 8.0;
    let damping = 0.9;
    let position_correction_factor = 0.5;
    
    // Create temporary arrays to store the velocity and position changes
    let mut dx_vel_change = vec![0.0f32; count];
    let mut dy_vel_change = vec![0.0f32; count];
    let mut dx_pos_change = vec![0.0f32; count];
    let mut dy_pos_change = vec![0.0f32; count];
    
    // Process all particles for potential collisions
    for i in 0..count {
        // Get current particle position
        let simd_index = i / simd_level;
        let local_index = i % simd_level;
        let px = x[simd_index].as_array()[local_index];
        let py = y[simd_index].as_array()[local_index];
        
        // Get current particle velocity
        let vx = x_vel[simd_index].as_array()[local_index];
        let vy = y_vel[simd_index].as_array()[local_index];
        
        // Get potential collision candidates
        let candidates = grid.get_potential_collisions(px, py);
        
        for &other_index in candidates.iter() {
            // Skip self-collision
            if i == other_index {
                continue;
            }
            
            // Skip if we've already processed this pair (to avoid double-counting)
            if other_index < i {
                continue;
            }
            
            // Get other particle position
            let other_simd_index = other_index / simd_level;
            let other_local_index = other_index % simd_level;
            let other_x = x[other_simd_index].as_array()[other_local_index];
            let other_y = y[other_simd_index].as_array()[other_local_index];
            
            // Get other particle velocity
            let other_vx = x_vel[other_simd_index].as_array()[other_local_index];
            let other_vy = y_vel[other_simd_index].as_array()[other_local_index];
            
            // Calculate distance
            let dx = px - other_x;
            let dy = py - other_y;
            let dist_sq = dx * dx + dy * dy;
            
            // Check for collision
            if dist_sq < COLLISION_RADIUS * COLLISION_RADIUS && dist_sq > 0.0001 {
                let dist = dist_sq.sqrt();
                let nx = dx / dist; // Normalized direction
                let ny = dy / dist;
                
                // Calculate overlap
                let overlap = COLLISION_RADIUS - dist;
                
                // Calculate relative velocity
                let rel_vx = vx - other_vx;
                let rel_vy = vy - other_vy;
                
                // Project relative velocity onto collision normal
                let rel_vel_along_normal = nx * rel_vx + ny * rel_vy;
                
                // Only apply repulsion if particles are moving toward each other
                if rel_vel_along_normal < 0.0 {
                    // Calculate impulse magnitude (with equal mass assumption)
                    let impulse = -(1.0 + damping) * rel_vel_along_normal / 2.0;
                    
                    // Apply impulse to both particles
                    dx_vel_change[i] += nx * impulse;
                    dy_vel_change[i] += ny * impulse;
                    dx_vel_change[other_index] -= nx * impulse;
                    dy_vel_change[other_index] -= ny * impulse;
                }
                
                // Apply additional repulsion force for very close particles
                let repulsion = repulsion_strength * overlap;
                dx_vel_change[i] += nx * repulsion;
                dy_vel_change[i] += ny * repulsion;
                dx_vel_change[other_index] -= nx * repulsion;
                dy_vel_change[other_index] -= ny * repulsion;
                
                // Position correction to prevent sinking
                let correction = overlap * position_correction_factor;
                dx_pos_change[i] += nx * correction;
                dy_pos_change[i] += ny * correction;
                dx_pos_change[other_index] -= nx * correction;
                dy_pos_change[other_index] -= ny * correction;
            }
        }
    }
    
    // Apply accumulated velocity and position changes
    for i in 0..count {
        let simd_index = i / simd_level;
        let local_index = i % simd_level;
        
        // Get current arrays from SIMD vectors
        let mut x_array = x[simd_index].as_array().clone();
        let mut y_array = y[simd_index].as_array().clone();
        let mut x_vel_array = x_vel[simd_index].as_array().clone();
        let mut y_vel_array = y_vel[simd_index].as_array().clone();
        
        // Apply velocity changes
        x_vel_array[local_index] += dx_vel_change[i];
        y_vel_array[local_index] += dy_vel_change[i];
        
        // Apply position changes
        x_array[local_index] += dx_pos_change[i];
        y_array[local_index] += dy_pos_change[i];
        
        // Update the SIMD vectors
        x[simd_index] = f32x32::from_array(x_array);
        y[simd_index] = f32x32::from_array(y_array);
        x_vel[simd_index] = f32x32::from_array(x_vel_array);
        y_vel[simd_index] = f32x32::from_array(y_vel_array);
    }
}

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

        // resolve_collisions(&mut x, &mut y, &mut x_vel, &mut y_vel, COUNT, SIMD_LEVEL);

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
