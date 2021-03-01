use std::error::Error;
use std::time::SystemTime;
use std::ops::{Add, Mul};

pub fn main(config: Config) -> Result<String, Box<dyn Error>> {
    let start_time = SystemTime::now().elapsed()?;

    let coords = calculate_sample_points(&config);

    let end_time = SystemTime::now().elapsed()?;
    println!("Time: {}", end_time.as_micros() - start_time.as_micros());
    Ok(String::from("hi"))
}

fn calculate_sample_points(config: &Config) -> Box<Vec<Vec<CNumber>>> {
    let step_size_x = 3.0 / config.width;
    let step_size_y = 2.0 / config.height;

    let offset_x = config.center.real - config.width / 2.0 * step_size_x;
    let offset_y = -(config.center.imag - config.height / 2.0 * step_size_y);

    let mut coords: Box<Vec<Vec<CNumber>>> =
        Box::from(vec![vec![CNumber::new(0.0, 0.0); config.width as usize]; config.height as usize]);

    for i in 0..config.width as usize {
        for j in 0..config.height as usize {
            coords[j][i].real = offset_x + step_size_x * i as f64;
            coords[j][i].imag = offset_y + step_size_y * j as f64;
        }
    }

    println!("{:?}", coords[0][0]);
    coords
}

fn check_mandelbrot(number: &CNumber, iter: i32, threshold: f64) -> i32 {
    let mut n = CNumber::new(0.0, 0.0);
    let c = number;

    n = n + c;

    for _ in 0..iter {
        n = n * n + c;
    }

    if n.real < threshold && n.imag < threshold {
        1
    } else {
        0
    }
}

fn draw(values: Box<Vec<Vec<i32>>>) -> String {
    let mut lines = vec![];

}

#[derive(Copy, Clone, Debug)]
struct CNumber {
    real: f64,
    imag: f64,
}

impl CNumber {
    fn new(real: f64, imag: f64) -> CNumber {
        CNumber { real, imag }
    }
}

impl Add for &CNumber {
    type Output = CNumber;

    fn add(&self, b: CNumber) -> Self::Output {
        let real = self.real + b.real;
        let imag = self.imag + b.imag;

        CNumber { real, imag }
    }
}

impl Mul for &CNumber {
    type Output = CNumber;

    fn mul(self, b: Self) -> Self::Output {
        let real = self.real * self.real - b.imag * b.imag;
        let imag = self.real * b.imag + b.real * self.imag;

        CNumber { real, imag }
    }
}


pub struct Config {
    quality: i32,
    width: f64,
    threshold: f64,
    //-- calculated
    height: f64,
    center: CNumber,
    iterations: i32,
}

impl Config {
    pub fn new(point_number: usize, quality: i32, width: i32, threshold: f32) -> Config {
        let height = width as f32 * 0.2;

        let interesting_points = vec![CNumber::new(-0.75, 0.0), CNumber::new(-0.77568377, 0.13646737)];
        let center = interesting_points[point_number];
        let iterations = match quality {
            0 => 20,
            1 => 50,
            2 => 100,
            3 => 500,
            4 => 1000,
            _ => quality
        };

        Config {
            quality,
            width: width as f64,
            height: height as f64,
            center,
            iterations,
            threshold: threshold as f64,
        }
    }
}

//#[cfg(tests)]
mod tests {
    use crate::{Config, calculate_sample_points};

    #[test]
    fn correct_size_points() {
        let config = Config::new(1, 0, 100, 0.0);

        let result = calculate_sample_points(&config);

        result[0][0];
        result[0][99];
    }
}