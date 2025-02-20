use std::simd::*;
use std::time::Instant;

use core::fmt::Formatter;
use std::fmt;

enum OrMore<T> {
    Value(T),
    More,
}

impl<T: fmt::Debug> fmt::Debug for OrMore<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            OrMore::Value(t) => fmt::Debug::fmt(t, f),
            OrMore::More => write!(f, "..."),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LimitedVec<T>(pub Vec<T>);
const LIMIT: usize = 4;
impl<T: fmt::Debug> fmt::Debug for LimitedVec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.len() <= LIMIT {
            f.debug_list().entries(self.0.iter().take(LIMIT)).finish()
        } else {
            f.debug_list()
                .entries(
                    self.0
                        .iter()
                        .take(LIMIT)
                        .map(OrMore::Value)
                        .chain(vec![OrMore::More].into_iter()),
                )
                .finish()
        }
    }
}

pub fn simd_bench() {
    const COUNT: usize = 10_000_000;

    {
        let x = [0.001f32; COUNT];
        let y = [0.001f32; COUNT];
        let mut sum: f32 = 0.0;

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-1 {:?} {}", time.elapsed(), sum);
    }

    {
        let x = [f32x4::splat(0.001); COUNT];
        let y = [f32x4::splat(0.001); COUNT];
        let mut sum = f32x4::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-4 {:?} {:?}", time.elapsed(), LimitedVec(sum.to_array().to_vec()));
    }

    {
        let x = [f32x8::splat(0.001); COUNT];
        let y = [f32x8::splat(0.001); COUNT];
        let mut sum = f32x8::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-8 {:?} {:?}", time.elapsed(), LimitedVec(sum.to_array().to_vec()));
    }

    {
        let x = [f32x16::splat(0.001); COUNT];
        let y = [f32x16::splat(0.001); COUNT];
        let mut sum = f32x16::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-16 {:?} {:?}", time.elapsed(), LimitedVec(sum.to_array().to_vec()));
    }

    {
        let x = [f32x32::splat(0.001); COUNT];
        let y = [f32x32::splat(0.001); COUNT];
        let mut sum = f32x32::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-32 {:?} {:?}", time.elapsed(), LimitedVec(sum.to_array().to_vec()));
    }

    {
        let x = [f32x64::splat(0.001); COUNT];
        let y = [f32x64::splat(0.001); COUNT];
        let mut sum = f32x64::splat(0.0f32);

        let time = Instant::now();

        for i in 0..COUNT {
            sum += x[i] * y[i];
        }

        println!("simd-64 {:?} {:?}", time.elapsed(), LimitedVec(sum.to_array().to_vec()));
    }
}
