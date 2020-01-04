use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use num::Complex;
use rand::distributions::Standard;
use rand::Rng;

fn bench_f32(
    c: &mut Criterion,
    title: &str,
    sizes: &mut dyn std::iter::Iterator<Item = usize>,
    forward: bool,
) {
    let mut group = c.benchmark_group(title);
    for size in sizes {
        let input = rand::thread_rng()
            .sample_iter(&Standard)
            .zip(rand::thread_rng().sample_iter(&Standard))
            .take(size)
            .map(|(x, y)| Complex::new(x, y))
            .collect::<Vec<_>>();

        // Fourier
        let fourier = fourier::create_fft_f32(size);
        group.bench_with_input(BenchmarkId::new("Fourier", size), &input, |b, i| {
            let mut input = Vec::new();
            input.extend_from_slice(i);
            b.iter(|| fourier.fft_in_place(input.as_mut()))
        });

        // RustFFT
        let rustfft = rustfft::FFTplanner::<f32>::new(!forward).plan_fft(size);
        group.bench_with_input(BenchmarkId::new("RustFFT", size), &input, |b, i| {
            let mut input = Vec::new();
            input.extend_from_slice(i);
            let mut output = vec![Complex::default(); input.len()];
            b.iter(|| rustfft.process(input.as_mut(), output.as_mut()))
        });

        use fftw::plan::C2CPlan;
        let mut fftw = fftw::plan::C2CPlan32::aligned(
            &[size],
            if forward {
                fftw::types::Sign::Forward
            } else {
                fftw::types::Sign::Backward
            },
            fftw::types::Flag::Measure,
        )
        .unwrap();
        group.bench_with_input(BenchmarkId::new("FFTW", size), &input, |b, i| {
            let mut input = fftw::array::AlignedVec::new(size);
            for (i, o) in i.iter().zip(input.as_slice_mut().iter_mut()) {
                *o = fftw::types::c32::new(i.re, i.im);
            }
            let mut output = fftw::array::AlignedVec::new(size);
            b.iter(|| {
                fftw.c2c(input.as_slice_mut(), output.as_slice_mut())
                    .unwrap()
            })
        });
    }
    group.finish();
}

fn bench_f32_pow2(c: &mut Criterion) {
    bench_f32(
        c,
        "FFT, f32, powers of two",
        &mut (8..10).map(|x| 2usize.pow(x)),
        true,
    );
}

fn bench_f32_prime(c: &mut Criterion) {
    bench_f32(
        c,
        "FFT, f32, primes",
        &mut [191, 439, 1013].iter().map(|x| *x),
        true,
    );
}

criterion_group!(benches, bench_f32_pow2, bench_f32_prime);
criterion_main!(benches);
