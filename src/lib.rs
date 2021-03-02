use std::error::Error;
use std::time::SystemTime;
use std::ops::{Add, Mul};

pub fn run(config: Config) -> Result<String, Box<dyn Error>> {
    check_mandelbrot(&CNumber::new(-2.17068377, -1.13646737), 100, 100.0);

    let start_time = SystemTime::now();
    //let start_time = start_time.as_millis();

    let coords = calculate_sample_points(&config);
    let result = check_whole_mandelbrot(&coords, config.iterations, config.threshold);
    let draw = draw(result);
    println!("{}", draw);

    println!("Time: {}ms", start_time.elapsed()?.as_millis());
    Ok(String::from("hi"))
}

fn calculate_sample_points(config: &Config) -> Vec<Vec<CNumber>> {
    let step_size_x = 3.0 / config.width;
    let step_size_y = 2.0 / config.height;

    let offset_x = config.center.real - config.width / 2.0 * step_size_x;
    let offset_y = -(config.center.imag - config.height / 2.0 * step_size_y) - 2.0;

    let mut coords: Vec<Vec<CNumber>> =
        vec![vec![CNumber::new(0.0, 0.0); config.width as usize]; config.height as usize];

    for i in 0..config.width as usize {
        for j in 0..config.height as usize {
            coords[j][i].real = offset_x + step_size_x * i as f64;
            coords[j][i].imag = offset_y + step_size_y * j as f64;
        }
    }

    println!("{:?}", coords[0][0]);
    coords
}

static HIGH: &str = "#";
static LOW: &str = " ";

fn check_whole_mandelbrot(nums: &Vec<Vec<CNumber>>, iter: i32, threshold: f64) -> Vec<Vec<&str>> {
    let height = nums.len();
    let width = nums[0].len();

    let mut result: Vec<Vec<&str>> = vec![vec![""; nums[0].len()]; nums.len()];

    for i in 0..height {
        for j in 0..width {
            result[i][j] = check_mandelbrot(&nums[i][j], iter, threshold);
        }
    }

    result
}

fn check_mandelbrot(number: &CNumber, iter: i32, threshold: f64) -> &str {
    let mut n = CNumber::new(0.0, 0.0);
    let c = number;

    n = n + *c;

    for _ in 0..iter {
        n = n * n + *c;

        if n.imag > threshold || n.real > threshold {
            return LOW;
        }
    }

    HIGH
}

fn draw(values: Vec<Vec<&str>>) -> String {
    let mut out = String::new();

    for line in values {
        for char in line {
            out += char;
        }
        out += "\n";
    }

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
            1 => 500,
            2 => 1000,
            3 => 5000,
            4 => 20000,
            _ => quality
        };

        Config {
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

        assert_eq!(check_mandelbrot(&CNumber::new(1.0, 1.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(2.0, 0.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(0.0, 0.0), iter, thr), HIGH);
        assert_eq!(check_mandelbrot(&CNumber::new(0.0, 3.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(0.8, 0.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(0.7, 0.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(0.7, 0.0), iter, thr), LOW);
        assert_eq!(check_mandelbrot(&CNumber::new(0.1, 0.1), iter, thr), HIGH);
        assert_eq!(check_mandelbrot(&CNumber::new(0.1, 0.1), iter, thr), HIGH);
        assert_eq!(check_mandelbrot(&CNumber::new(-2.17068377, -1.13646737), iter, thr), LOW); //CNumber { real: -2.17068377, imag: -1.13646737 }
    }

    #[test]
    fn draw_test() {
        let vector = vec![vec!["a", "b", "c", "d", "e"]; 2];
        let out = draw(vector);
        println!("{}", out);
        assert_eq!(out, "abcde
abcde
")
    }
}