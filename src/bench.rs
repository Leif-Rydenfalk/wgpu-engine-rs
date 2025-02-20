use std::simd::{f32x4, f32x8};
use std::time::Instant;

pub fn simd_bench() {
    {
        const COUNT: usize = 1_000_000;
        let x = [2.0f32; COUNT];
        let y = [2.0f32; COUNT];
        let mut sum: f32 = 0.0;

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("{:?} {}", time.elapsed(), sum);
    }

    {
        const COUNT: usize = 1_000_000;
        let x = [f32x4::splat(2.0f32); COUNT];
        let y = [f32x4::splat(2.0f32); COUNT];
        let mut sum = f32x4::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("{:?} {:?}", time.elapsed(), sum);
    }

    {
        const COUNT: usize = 1_000_000;
        let x = [f32x8::splat(2.0f32); COUNT];
        let y = [f32x8::splat(2.0f32); COUNT];
        let mut sum = f32x8::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("{:?} {:?}", time.elapsed(), sum);
    }
}

            // let mut y_vel_f32x8 =  [f32x8::new([2.0f32; 8]); COUNT]; 
            // for i in 0..COUNT / 2 {
            //     y_vel_f32x8[i] = f32x8::new([y_vel[i].as_array_ref(), y_vel[i + 1].as_array_ref()]);
                
            // }