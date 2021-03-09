use std::env::Args;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Mul};
use std::time::{Duration, SystemTime};

use image::{ImageBuffer, Rgb, RgbImage, ImageResult};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let start_time = SystemTime::now();
    let debug = config.debug;

    let height = config.width * 2.0 / 3.0;

    let mut handles = vec![];
    let result: Arc<Mutex<Vec<Vec<u32>>>> = Arc::new(Mutex::new(vec![vec![0; config.width as usize]; height as usize]));

    let thread_size = 10;

    for i in 0..thread_size {
        let mut result = Arc::clone(&result);
        let id = i.clone();
        let config = config.clone();
        let thread_size = thread_size;
        let handle = thread::spawn(move || {
            check_part_of_mandelbrot(&mut result, &config, thread_size, id)
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    if config.is_image {
        create_image(&result.lock().unwrap(), config.iterations, &*config.image_path)?
    }

    if debug { println!("calculated in: {}", format_time(start_time.elapsed()?)); }

    if debug { println!("Total Time: {}", format_time(start_time.elapsed()?)); }
    Ok(())
}


fn check_part_of_mandelbrot(vec: &mut Arc<Mutex<Vec<Vec<u32>>>>, config: &Config, parts: u32, id: u32) {
    let height = config.width * 2.0 / 3.0;

    let step_size = CNumber {
        real: 3.0 / config.width,
        imag: 2.0 / height,
    };
    let offset = CNumber {
        real: config.center.real - config.width / 2.0 * step_size.real,
        imag: -(config.center.imag - height / 2.0 * step_size.imag) - 2.0,
    };

    let part_height = height as u32 / parts;
    let start_index = part_height * id;
    let end_index = part_height * (id + 1);

    for i in start_index as usize..end_index as usize {
        for j in 0..config.width as usize {
            vec.lock().unwrap()[i][j] = check_mandelbrot(j, i, config, &offset, &step_size);
        }
    }
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

fn create_image(values: &Vec<Vec<u32>>, iterations: u32, path: &str) -> ImageResult<()> {
    let w = values[0].len() as u32;
    let h = values.len() as u32;

    let mut image: RgbImage = ImageBuffer::new(w as u32, h as u32);

    for y in 0..h {
        for x in 0..w {
            let val = values[y as usize][x as usize];
            *image.get_pixel_mut(x, y) = get_color_for_pixel(val as f32, iterations as f32);
        }
    }

    image.save(path)
}


fn get_color_for_pixel(value: f32, iter: f32) -> Rgb<u8> {
    let multiplier: f32 = 1.0 - (value * value).min(iter) / iter;
    let i: u8 = (255 as f32 * multiplier) as u8;
    image::Rgb([i, i, i])
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
        return format!("{}ns", d.as_nanos());
    }
    if d.as_millis() < 10 {
        return format!("{}μs", d.as_micros());
    }
    if d.as_secs() < 10 {
        return format!("{}ms", d.as_millis());
    }

    let ms = d.as_millis() % 1000;
    let secs = d.as_secs();

    if secs < 60 {
        return format!("{}s {}ms", secs, ms);
    }

    let mins = secs / 60;
    let secs = secs % 60;

    if mins < 60 {
        return format!("{}m {}s {}ms", mins, secs, ms);
    }
    let hours = mins / 60;
    let mins = mins % 60;
    format!("{}h {}m {}s {}ms", hours, mins, secs, ms)
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

#[derive(Clone)]
pub struct Config {
    width: f64,
    threshold: f64,
    //-- calculated
    center: CNumber,
    iterations: u32,
    is_image: bool,
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
            Some("debug") | Some("dbg") =>
                self.debug = true,
            _ => return Err(Box::new(PropertyError { msg: format!("Property not found: {}", key.unwrap_or_else(|| "")) }))
        }

        Ok(())
    }


    pub fn default() -> Config {
        Config::new(1, 3, 100, 100.0, false, String::from("img.png"), false)
    }

    pub fn new(point_number: usize, quality: i32, width: i32, threshold: f32, is_image: bool, image_path: String, debug: bool) -> Config {
        let interesting_points = vec![CNumber::new(-0.75, 0.0), CNumber::new(-0.77568377, 0.13646737)];
        let center = interesting_points[point_number];
        let iterations = config_iter_from_quality(quality);

        Config {
            width: width as f64,
            center,
            iterations,
            threshold: threshold as f64,
            is_image,
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
    use crate::{calculate_sample_points, check_mandelbrot, CNumber, Config, draw, HIGH, LOW};

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