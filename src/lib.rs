use std::error::Error;
use std::time::SystemTime;
use std::ops::{Add, Mul};
use std::env::Args;
use std::fmt::{Display, Formatter, Pointer};

pub fn run(config: Config) -> Result<String, Box<dyn Error>> {
    println!("w={}", config.width);

    let start_time = SystemTime::now();

    let coords = calculate_sample_points(&config);
    println!("coords done after: {}μs", start_time.elapsed()?.as_micros());

    let result = check_whole_mandelbrot(&coords, config.iterations, config.threshold);
    let draw = draw(result, config.iterations);
    println!("{}", draw);

    println!("Total Time: {}ms", start_time.elapsed()?.as_millis());
    Ok(String::from("hi"))
}

fn calculate_sample_points(config: &Config) -> Vec<Vec<CNumber>> {
    let start_time = SystemTime::now();

    let height = config.width as f64 * 0.2;

    let step_size_x = 3.0 / config.width;
    let step_size_y = 2.0 / height;

    let offset_x = config.center.real - config.width / 2.0 * step_size_x;
    let offset_y = -(config.center.imag - height / 2.0 * step_size_y) - 2.0;

    let mut coords: Vec<Vec<CNumber>> =
        vec![vec![CNumber::new(0.0, 0.0); config.width as usize]; height as usize];

    println!("Allocated sample vector after {}μs", start_time.elapsed().unwrap().as_micros());

    for i in 0..config.width as usize {
        for j in 0..height as usize {
            coords[j][i].real = offset_x + step_size_x * i as f64;
            coords[j][i].imag = offset_y + step_size_y * j as f64;
        }
    }

    coords
}

fn check_whole_mandelbrot(nums: &Vec<Vec<CNumber>>, iter: i32, threshold: f64) -> Vec<Vec<i32>> {
    let start_time = SystemTime::now();
    println!("Started calculating");

    let height = nums.len();
    let width = nums[0].len();

    let mut result: Vec<Vec<i32>> = vec![vec![0; nums[0].len()]; nums.len()];


    for i in 0..height {
        for j in 0..width {
            result[i][j] = check_mandelbrot(&nums[i][j], iter, threshold);
        }
    }

    println!("Calculated results after {}ms", start_time.elapsed().unwrap().as_millis());


    result
}

fn check_mandelbrot(number: &CNumber, iter: i32, threshold: f64) -> i32 {
    //let start_time = SystemTime::now();

    let mut n = CNumber::new(0.0, 0.0);
    let c = number;

    n = n + *c;

    for i in 0..iter {
        n = n * n + *c;

        if n.imag > threshold || n.real > threshold {
            return i;
        }
    }

    iter
}

static HIGH: &str = "#";
static LOW: &str = " ";

fn draw(values: Vec<Vec<i32>>, iterations: i32) -> String {
    let start_time = SystemTime::now();
    let mut out = String::new();

    for line in values {
        for char in line {
            out += if char < iterations { LOW } else { HIGH };
        }
        out += "\n";
    }

    println!("Finished drawing after {}μs", start_time.elapsed().unwrap().as_micros());

    out
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct CNumber {
    real: f64,
    imag: f64,
}

impl CNumber {
    fn new(real: f64, imag: f64) -> CNumber {
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
        //(a+bi)(c+di) = (ac−bd) + (ad+bc)i
        let real = self.real * b.real - self.imag * b.imag; //ac−bd
        let imag = self.real * b.imag + self.imag * b.real; //ad+bc

        CNumber { real, imag }
    }
}


pub struct Config {
    width: f64,
    threshold: f64,
    //-- calculated
    center: CNumber,
    iterations: i32,
}

impl Config {
    pub fn from(args: Args) -> Result<Config, Box<dyn Error>> {
        let mut config = Config::default();

        for arg in args {
            if arg.contains("=") {
                let mut split = arg.split("=");
                let key = split.next();
                let value = split.next();

                let value_f64: f64 = value.ok_or_else(||PropertyError {msg: format!("Error while parsing argument {}", arg)})?.parse()?;
                config.set_value(key, value_f64);
                println!("k={}, v={}, v64={}", key.unwrap(), value.unwrap(), value_f64);
            }
        }

        Ok(config)
    }

    fn set_value(&mut self, key: Option<&str>, value: f64) -> Result<(), Box<dyn Error>> {
        println!("setting arg value");
        match key {
            Some("iter") | Some("iterations") =>
                self.iterations = value as i32,
            Some("thres") | Some("threshold") =>
                self.threshold = value,
            Some("w") | Some("width") =>
                self.width = value,
            Some("quality") | Some("q") =>
                self.iterations = value as i32,
            _ => return Err(Box::new(PropertyError { msg: format!("Property not found: {}", key.unwrap_or_else(|| "")) }))
        }

        Ok(())
    }

    pub fn default() -> Config {
        Config::new(1, 3, 100, 100.0)
    }

    pub fn new(point_number: usize, quality: i32, width: i32, threshold: f32) -> Config {
        let interesting_points = vec![CNumber::new(-0.75, 0.0), CNumber::new(-0.77568377, 0.13646737)];
        let center = interesting_points[point_number];
        let iterations = config_iter_from_quality(quality);

        Config {
            width: width as f64,
            center,
            iterations,
            threshold: threshold as f64,
        }
    }
}

fn config_iter_from_quality(quality: i32) -> i32 {
    match quality {
        0 => 20,
        1 => 500,
        2 => 1000,
        3 => 5000,
        4 => 20000,
        _ => quality
    }
}

#[derive(Debug)]
struct PropertyError {
    msg: String
}

impl Display for PropertyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

impl Error for PropertyError {}

#[cfg(tests)]
mod tests {
    use crate::{CNumber, calculate_sample_points, check_mandelbrot, draw, HIGH, LOW, Config};

    #[test]
    fn cnumber_add_test() {
        let a = CNumber::new(1.0, 1.0);
        let b = CNumber::new(1.0, 1.0);
        assert_eq!(a + b, CNumber::new(2.0, 2.0));

        let a = CNumber::new(0.0, 0.0);
        let b = CNumber::new(0.0, -1.0);
        assert_eq!(a + b, CNumber::new(0.0, -1.0));

        let a = CNumber::new(5.0, -13.0);
        let b = CNumber::new(10.0, 5.0);
        assert_eq!(a + b, CNumber::new(15.0, -8.0))
    }

    #[test]
    fn cnumber_mul_test() {
        let a = CNumber::new(1.0, 2.0);
        let b = CNumber::new(3.0, 4.0);
        assert_eq!(a * b, CNumber::new(-5.0, 10.0));
    }

    #[test]
    fn correct_size_points() {
        let config = Config::new(1, 0, 100, 0.0);

        let result = calculate_sample_points(&config);

        result[0][0];
        result[0][99];
    }

    #[test]
    fn check_mandelbrot_test() {
        let iter = 1000;
        let thr = 100.0;

        assert!(check_mandelbrot(&CNumber::new(1.0, 1.0), iter, thr) < iter);
        assert!(check_mandelbrot(&CNumber::new(2.0, 0.0), iter, thr) < iter);
        assert_eq!(check_mandelbrot(&CNumber::new(0.0, 0.0), iter, thr), iter);
        assert!(check_mandelbrot(&CNumber::new(0.0, 3.0), iter, thr) < iter);
        assert!(check_mandelbrot(&CNumber::new(0.8, 0.0), iter, thr) < iter);
        assert!(check_mandelbrot(&CNumber::new(0.7, 0.0), iter, thr) < iter);
        assert!(check_mandelbrot(&CNumber::new(0.7, 0.0), iter, thr) < iter);
        assert_eq!(check_mandelbrot(&CNumber::new(0.1, 0.1), iter, thr), iter);
        assert_eq!(check_mandelbrot(&CNumber::new(0.1, 0.1), iter, thr), iter);
        assert!(check_mandelbrot(&CNumber::new(-2.17068377, -1.13646737), iter, thr) < iter); //CNumber { real: -2.17068377, imag: -1.13646737 }
    }

    #[test]
    fn draw_test() {
        let vector = vec![vec![0, 0, 10, 10, 10]; 2];
        let out = draw(vector, 10);
        println!("{}", out);
        assert_eq!(out, "  ###
  ###
")
    }
}