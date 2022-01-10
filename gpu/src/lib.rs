#![no_std]
#![feature(register_attr)]
#![register_attr(nvvm_internal)]
#![allow(improper_ctypes_definitions)]

use core::ops::{Add, Mul};
use cuda_std::kernel;
use cuda_std::thread;
use cuda_std::vek::Vec2;

#[derive(Copy, Clone, PartialEq)]
pub struct CNumber {
    pub real: f64,
    pub imag: f64,
}

#[derive(Copy, Clone)]
pub struct Cfg {
    pub start: CNumber,
    pub end: CNumber,
    pub height: u32,
    pub width: u32,
    pub iterations: u32,
    pub threshold: u32,
}

#[kernel]
pub unsafe fn mandelbrot(
    start_real: f64,
    start_imag: f64,
    end_real: f64,
    end_imag: f64,
    height: u32,
    width: u32,
    iterations: u32,
    threshold: u32,
    out: *mut u32,
) {
    let cfg = Cfg {
        start: CNumber::new(start_real, start_imag),
        end: CNumber::new(end_real, end_imag),
        height,
        width,
        iterations,
        threshold
    };

    let idx = thread::index_2d();

    if idx.x >= width || idx.y >= height {
        return;
    }

    let ret = check_part_of_mandelbrot(cfg, idx);

    let offset = idx.x + (idx.y * cfg.width);
    out.add(offset as usize).write(ret)
}

fn check_part_of_mandelbrot(cfg: Cfg, id: Vec2<u32>) -> u32 {
    let x_step_size = (cfg.end.real - cfg.start.real) / cfg.width as f64;
    let y_step_size = (cfg.end.imag - cfg.start.imag) / cfg.height as f64;

    let x = x_step_size * id.x as f64;
    let y = y_step_size * id.y as f64;

    let sample_pos = CNumber {
        real: cfg.start.real + x,
        imag: cfg.start.imag + y,
    };

    check_mandelbrot(sample_pos, cfg.iterations, cfg.threshold)
}

fn check_mandelbrot(sample_pos: CNumber, iterations: u32, threshold: u32) -> u32 {
    let mut n = CNumber::new(0.0, 0.0);
    let c = sample_pos;

    n = n + c;

    for i in 0..iterations {
        n = n * n + c;

        if n.imag > threshold as f64 || n.real > threshold as f64 {
            return i;
        }
    }

    iterations
}

impl CNumber {
    pub fn new(real: f64, imag: f64) -> CNumber {
        CNumber { real, imag }
    }
}

impl Add for CNumber {
    type Output = CNumber;

    fn add(self, b: CNumber) -> Self::Output {
        let real = self.real + b.real;
        let imag = self.imag + b.imag;

        CNumber { real, imag }
    }
}

impl Mul for CNumber {
    type Output = CNumber;

    fn mul(self, b: Self) -> Self::Output {
        let real = self.real * b.real - self.imag * b.imag;
        let imag = self.real * b.imag + self.imag * b.real;

        CNumber { real, imag }
    }
}
