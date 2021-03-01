use std::error::Error;
use std::time::SystemTime;

pub fn main(config: Config) -> Result<String, Box<dyn Error>> {
    let start_time = SystemTime::now().elapsed()?;


    let end_time = SystemTime::now().elapsed()?;
    println!("Time: {}", end_time.as_micros() - start_time.as_micros());
    Ok(String::from("hi"))
}

#[derive(Copy, Clone)]
struct CNumber {
    real: f64,
    imag: f64,
}

impl CNumber {
    fn new(real: f64, imag: f64) -> CNumber {
        CNumber { real, imag }
    }

    fn add_mut(&mut self, other: CNumber) {
        self.real += other.real;
        self.imag += other.imag;
    }

    fn mul_mut(&mut self, other: CNumber) {
        self.real = self.real * self.real - other.imag * other.imag;
        self.imag = self.real * other.imag + other.real * self.imag;
    }
}

fn add_c(a: &CNumber, b: &CNumber) -> CNumber {
    let real = a.real + b.real;
    let imag = a.imag + b.imag;

    CNumber { real, imag }
}

fn mul_c(a: &CNumber, b: &CNumber) -> CNumber {
    let real = a.real * a.real - b.imag * b.imag;
    let imag = a.real * b.imag + b.real * a.imag;

    CNumber { real, imag }
}

pub struct Config {
    quality: i32,
    width: i32,
    threshold: f32,
    //-- calculated
    height: i32,
    center: CNumber,
    iterations: i32,
}

impl Config {
    pub fn new(point_number: usize, quality: i32, width: i32, threshold: f32) -> Config{
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
            quality, width, height: height as i32,
            center, iterations, threshold,
        }
    }
}