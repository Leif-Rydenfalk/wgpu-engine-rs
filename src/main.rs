use std::thread;
use std::time::Instant;
use wide::{f32x4, f32x8};

// fn simd_bench() {
//     {
//         const COUNT: usize = 1_000_000;
//         let x = [2.0f32; COUNT];
//         let y = [2.0f32; COUNT];
//         let mut sum: f32 = 0.0;

//         let time = Instant::now();

//         for i in 0..COUNT {
//             sum += x[i] * y[i];
//         }

//         println!("{:?} {}", time.elapsed(), sum);
//     }

//     {
//         const COUNT: usize = 1_000_000;
//         let x = [f32x4::new([2.0f32; 4]); COUNT];
//         let y = [f32x4::new([2.0f32; 4]); COUNT];
//         let mut sum = f32x4::new([0.0f32; 4]);

//         let time = Instant::now();

//         for i in 0..COUNT {
//             sum += x[i] * y[i];
//         }

//         println!("{:?} {}", time.elapsed(), sum);
//     }

//     {
//         const COUNT: usize = 1_000_000;
//         let x = [f32x8::new([2.0f32; 8]); COUNT];
//         let y = [f32x8::new([2.0f32; 8]); COUNT];
//         let mut sum = f32x8::new([0.0f32; 8]);

//         let time = Instant::now();

//         for i in 0..COUNT {
//             sum += x[i] * y[i];
//         }

//         println!("{:?} {}", time.elapsed(), sum);
//     }
// }

fn sim() {
    const FRAMES: u32 = 100;

    {
        const COUNT: usize = 1_000_000;
        let mut x = [f32x4::new([2.0f32; 4]); COUNT];
        let mut x_vel = [f32x4::new([2.0f32; 4]); COUNT];

        let mut y = [f32x4::new([2.0f32; 4]); COUNT];
        let mut y_vel = [f32x4::new([2.0f32; 4]); COUNT];

        let time = Instant::now();

        let gravity: f32x4 = f32x4::new([0.1f32; 4]);
        let dt = f32x4::new([0.1f32; 4]);

        for _ in 0..FRAMES {
            // let mut y_vel_f32x8 =  [f32x8::new([2.0f32; 8]); COUNT]; 
            // for i in 0..COUNT / 2 {
            //     y_vel_f32x8[i] = f32x8::new([y_vel[i].as_array_ref(), y_vel[i + 1].as_array_ref()]);
                
            // }
            
            for i in 0..COUNT {
                y_vel[i] += gravity * dt;
            }

            for i in 0..COUNT {
                x[i] += x_vel[i] * dt;
                y[i] += y_vel[i] * dt;
            }
        }

        println!("{:?} {} {}", time.elapsed() / FRAMES, x[0], y[0]);
    }
}

fn main() {
    const STACK_SIZE: usize = 128 * 1_000_000;
    let sim_thread = thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(sim)
        .unwrap();
    sim_thread.join().unwrap();
}
