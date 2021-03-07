use std::error::Error;
use std::time::{SystemTime, Duration};
use std::ops::{Add, Mul};
use std::env::Args;
use std::fmt::{Display, Formatter};
use image::{RgbImage, ImageBuffer, ImageResult};
use std::io::{self, Write};

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let start_time = SystemTime::now();
    let debug = config.debug;

    let result = check_whole_mandelbrot(&config);

    if debug { println!("calculated after: {}", format_time(start_time.elapsed()?)); }

    if config.is_image {
        create_image(&result, config.iterations, "img.png")?;
        if debug { println!("image created after: {}", format_time(start_time.elapsed()?)); }
    }
    if config.is_console {
        let draw = draw(&result, config.iterations);
        println!("{}", draw);
        if debug { println!("drawn after: {}", format_time(start_time.elapsed()?)); }
    }

    if debug { println!("Total Time: {}", format_time(start_time.elapsed()?)); }
    Ok(())
}


fn check_whole_mandelbrot(config: &Config) -> Vec<Vec<u32>> {
    let height = if config.is_image {
        config.width * 2.0 / 3.0
    } else {
        config.width * 0.2
    };

    let step_size = CNumber {
        real: 3.0 / config.width,
        imag: 2.0 / height,
    };
    let offset = CNumber {
        real: config.center.real - config.width / 2.0 * step_size.real,
        imag: -(config.center.imag - height / 2.0 * step_size.imag) - 2.0,
    };

    let mut result: Vec<Vec<u32>> = vec![vec![0; config.width as usize]; height as usize];

    for i in 0..height as usize {
        for j in 0..config.width as usize {
            result[i][j] = check_mandelbrot(j, i, config, &offset, &step_size);
        }

        if config.debug {
            let progress = i as f64 / height;
            print!("\r{:.2}% {}", progress * 100.0, progress_bar(progress));
            let _ = io::stdout().flush();
        }
    }

    if config.debug {
        println!("\r100.00% {}", progress_bar(1.0));
    }

    result
}

fn check_mandelbrot(x: usize, y: usize, config: &Config, offset: &CNumber, step_size: &CNumber) -> u32 {
    let sample_pos = CNumber {
        real: offset.real + step_size.real * x as f64,
        imag: offset.imag + step_size.imag * y as f64,
    };

    let mut n = CNumber::new(0.0, 0.0);
    let c = sample_pos;

    n = n + c;

    for i in 0..config.iterations {
        n = n * n + c;

        if n.imag > config.threshold || n.real > config.threshold {
            return i;
        }
    }

    config.iterations
}

static HIGH: &str = "#";
static LOW: &str = " ";

fn draw(values: &Vec<Vec<u32>>, iterations: u32) -> String {
    let mut out = String::new();

    for line in values {
        for char in line {
            out += if char < &iterations { LOW } else { HIGH };
        }
        out += "\n";
    }

    out
}

fn create_image(values: &Vec<Vec<u32>>, iterations: u32, path: &str) -> ImageResult<()> {
    let w = values[0].len() as u32;
    let h = values.len() as u32;

    let mut image: RgbImage = ImageBuffer::new(w as u32, h as u32);

    for y in 0..h {
        for x in 0..w {
            let val = values[y as usize][x as usize];
            *image.get_pixel_mut(x, y) = if val < iterations { image::Rgb([255, 255, 255]) } else { image::Rgb([0, 0, 0]) };
        }
    }

    image.save(path)
}

static BAR_SIZE: usize = 50;

fn progress_bar(progress: f64) -> String {
    let mut bar = String::from("[");
    let bar_amount = (BAR_SIZE as f64 * progress).round() as usize;

    bar.push_str(&*"#".repeat(bar_amount));
    bar.push_str(&*"-".repeat(BAR_SIZE - bar_amount));

    bar.push(']');
    bar
}

fn format_time(d: Duration) -> String {
    if d.as_micros() < 10 {
        format!("{}ns", d.as_nanos())
    } else if d.as_millis() < 10 {
        format!("{}μs", d.as_micros())
    } else if d.as_secs() < 10 {
        format!("{}ms", d.as_millis())
    } else {
        let secs = d.as_secs();

        if secs < 60 {
            format!("{}s", secs)
        } else {
            let mins = secs / 60;
            let secs = secs % 60;

            if mins < 60 {
                format!("{}m {}s", mins, secs)
            } else {
                let hours = mins / 60;
                let mins = mins % 60;
                format!("{}h {}m {}s", hours, mins, secs)
            }
        }
    }
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
        let real = self.real * b.real - self.imag * b.imag;
        let imag = self.real * b.imag + self.imag * b.real;

        CNumber { real, imag }
    }
}


pub struct Config {
    width: f64,
    threshold: f64,
    //-- calculated
    center: CNumber,
    iterations: u32,
    is_image: bool,
    is_console: bool,
    image_path: String,
    debug: bool,
}

impl Config {
    pub fn from(args: Args) -> Result<Config, Box<dyn Error>> {
        let mut config = Config::default();

        for arg in args.into_iter().skip(1) {
            let mut split = arg.split("=");
            let key = split.next();
            let value = split.next();

            match value {
                None => config.set_value_flag(key)?,
                Some(_) => config.set_value_value(key, value, &arg)?
            };
        }

        if !config.is_image {
            config.is_console = true;
        }

        Ok(config)
    }

    fn set_value_value(&mut self, key: Option<&str>, value: Option<&str>, arg: &String) -> Result<(), Box<dyn Error>> {
        let val = value.ok_or_else(|| PropertyError { msg: format!("Error while parsing argument {}", arg) })?;

        return match key {
            Some("path") | Some("p") => {
                self.image_path = String::from(val);
                Ok(())
            }
            _ => {
                let value_f64: f64 = val.parse()?;
                self.set_value_f64(key, value_f64)
            }
        };
    }

    fn set_value_f64(&mut self, key: Option<&str>, value: f64) -> Result<(), Box<dyn Error>> {
        match key {
            Some("iter") | Some("iterations") =>
                self.iterations = value as u32,
            Some("thres") | Some("threshold") =>
                self.threshold = value,
            Some("w") | Some("width") =>
                self.width = value,
            Some("quality") | Some("q") =>
                self.iterations = value as u32,

            _ => return Err(Box::new(PropertyError { msg: format!("Property not found: {}", key.unwrap_or_else(|| "")) }))
        }

        Ok(())
    }

    fn set_value_flag(&mut self, key: Option<&str>) -> Result<(), Box<dyn Error>> {
        match key {
            Some("img") | Some("image") =>
                self.is_image = true,
            Some("console") | Some("cli") =>
                self.is_console = true,
            Some("debug") | Some("dbg") =>
                self.debug = true,
            _ => return Err(Box::new(PropertyError { msg: format!("Property not found: {}", key.unwrap_or_else(|| "")) }))
        }

        Ok(())
    }


    pub fn default() -> Config {
        Config::new(1, 3, 100, 100.0, false, String::from("img.png"), false, false)
    }

    pub fn new(point_number: usize, quality: i32, width: i32, threshold: f32, is_image: bool, image_path: String, is_console: bool, debug: bool) -> Config {
        let interesting_points = vec![CNumber::new(-0.75, 0.0), CNumber::new(-0.77568377, 0.13646737)];
        let center = interesting_points[point_number];
        let iterations = config_iter_from_quality(quality);

        Config {
            width: width as f64,
            center,
            iterations,
            threshold: threshold as f64,
            is_image,
            is_console,
            image_path,
            debug,
        }
    }
}

fn config_iter_from_quality(quality: i32) -> u32 {
    match quality {
        0 => 20,
        1 => 500,
        2 => 1000,
        3 => 5000,
        4 => 20000,
        _ => quality.abs() as u32
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
    fn draw_test() {
        let vector = vec![vec![0, 0, 10, 10, 10]; 2];
        let out = draw(vector, 10);
        println!("{}", out);
        assert_eq!(out, "  ###
  ###
")
    }
}