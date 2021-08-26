mod input;

use core::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Default random data size.
const COUNT: usize = 1000;

// ALGORITHMS

fn standard_div(v: u32) -> (u32, u32) {
    let x = v / 100;
    let y = v % 100;
    (x, y)
}

fn fast_div(v: u32) -> (u32, u32) {
    let divisor = 100;
    let max_precision = 14;
    let additional_precision = 5;

    let left_end = (((1 << (max_precision + additional_precision)) + divisor - 1) / divisor) as u32;
    let quotient = (v * left_end) >> (max_precision + additional_precision);
    let remainder = v - divisor * quotient;

    (quotient, remainder)
}

// GENERATOR

macro_rules! generator {
    (@div $group:ident, $name:expr, $iter:expr, $div:ident) => {{
        $group.bench_function($name, |bench| {
            bench.iter(|| {
                $iter.for_each(|&x| {
                    black_box($div(x));
                })
            })
        });
    }};

    ($group:ident, $name:literal, $iter:expr) => {{
        generator!(@div $group, concat!($name, "_standard_div"), $iter, standard_div);
        generator!(@div $group, concat!($name, "_fast_div"), $iter, fast_div);
    }};
}

// BENCHES

macro_rules! bench {
    ($fn:ident, $name:literal, $strategy:expr) => {
        fn $fn(criterion: &mut Criterion) {
            let mut group = criterion.benchmark_group($name);
            group.measurement_time(Duration::from_secs(5));
            let seed = fastrand::u64(..);

            let data: Vec<u32> = input::from_random::<u32>($strategy, COUNT, seed)
                .iter()
                .map(|x| x.parse::<u32>().unwrap())
                .collect();

            generator!(group, $name, data.iter());
        }
    };
}

bench!(uniform, "random:uniform", input::RandomGen::Uniform);

criterion_group!(uniform_benches, uniform);
criterion_main!(uniform_benches);
