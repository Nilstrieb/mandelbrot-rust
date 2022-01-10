#![allow(dead_code)]

use std::env::Args;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::{Duration, SystemTime};

use cust::prelude::*;
use gpu::{CNumber, Cfg};
use image::{ImageBuffer, ImageResult, Rgb, RgbImage};

static PTX: &str = include_str!("../target/gpu.ptx");

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let start_time = SystemTime::now();

    let debug = config.debug;
    let height = config.width * 2.0 / 3.0;

    let amount = (height * config.width) as usize;

    let _ctx = cust::quick_init()?;

    let module = Module::from_str(PTX)?;
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

    let mut out = vec![0u32; amount];
    let mut out_buf = out.as_slice().as_dbuf()?;

    let func = module.get_function("mandelbrot")?;

    let (_, block_size) = func.suggested_launch_configuration(amount, 0.into())?;

    let grid_size = (amount as u32 + block_size - 1) / block_size;

    let cfg = Cfg {
        start: CNumber::new(-0.5, 0.5),
        end: CNumber::new(0.5, 0.5),
        height: 500,
        width: 500,
        iterations: 1000,
        threshold: 100,
    };

    unsafe {
        launch!(
            func<<<grid_size, block_size, 0, stream>>>(
                cfg.start.real,
                cfg.start.imag,
                cfg.end.real,
                cfg.end.imag,
                cfg.height,
                cfg.width,
                cfg.iterations,
                cfg.threshold,
                out_buf.as_device_ptr(),
            )
        )?;
    }

    stream.synchronize()?;

    out_buf.copy_to(&mut out)?;

    create_image(&out, cfg, "gpu.png")?;

    // now do things with out
    println!("expected {}, got {} numbers!", amount, out.len());

    if debug {
        println!("calculated in: {}", format_time(start_time.elapsed()?));
    }

    if debug {
        println!("Total Time: {}", format_time(start_time.elapsed()?));
    }
    Ok(())
}

fn create_image(values: &[u32], cfg: Cfg, path: &str) -> ImageResult<()> {

    let mut image: RgbImage = ImageBuffer::new(cfg.width, cfg.height);

    for y in 0..cfg.height {
        for x in 0..cfg.width {
            let val = values[(x + y * cfg.width) as usize];
            *image.get_pixel_mut(x, y) = get_color_for_pixel(val as f32, cfg.iterations as f32);
        }
    }

    image.save(path)
}

fn get_color_for_pixel(value: f32, iter: f32) -> Rgb<u8> {
    let multiplier: f32 = 1.0 - (value * value).min(iter) / iter;
    let i: u8 = (255 as f32 * multiplier) as u8;
    image::Rgb([i, i, i])
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
                Some(_) => config.set_value_value(key, value, &arg)?,
            };
        }

        Ok(config)
    }

    fn set_value_value(
        &mut self,
        key: Option<&str>,
        value: Option<&str>,
        arg: &String,
    ) -> Result<(), Box<dyn Error>> {
        let val = value.ok_or_else(|| PropertyError {
            msg: format!("Error while parsing argument {}", arg),
        })?;

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
            Some("iter") | Some("iterations") => self.iterations = value as u32,
            Some("thres") | Some("threshold") => self.threshold = value,
            Some("w") | Some("width") => self.width = value,
            Some("quality") | Some("q") => self.iterations = value as u32,

            _ => {
                return Err(Box::new(PropertyError {
                    msg: format!("Property not found: {}", key.unwrap_or_else(|| "")),
                }))
            }
        }

        Ok(())
    }

    fn set_value_flag(&mut self, key: Option<&str>) -> Result<(), Box<dyn Error>> {
        match key {
            Some("img") | Some("image") => self.is_image = true,
            Some("debug") | Some("dbg") => self.debug = true,
            _ => {
                return Err(Box::new(PropertyError {
                    msg: format!("Property not found: {}", key.unwrap_or_else(|| "")),
                }))
            }
        }

        Ok(())
    }

    pub fn default() -> Config {
        Config::new(1, 3, 100, 100.0, false, String::from("img.png"), false)
    }

    pub fn new(
        point_number: usize,
        quality: i32,
        width: i32,
        threshold: f32,
        is_image: bool,
        image_path: String,
        debug: bool,
    ) -> Config {
        let interesting_points = vec![
            CNumber::new(-0.75, 0.0),
            CNumber::new(-0.77568377, 0.13646737),
        ];
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
        _ => quality.abs() as u32,
    }
}

#[derive(Debug)]
struct PropertyError {
    msg: String,
}

impl Display for PropertyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.msg)
    }
}

impl Error for PropertyError {}
