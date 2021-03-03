use mandelbrot_set::Config;
use std::env;

fn main() {
    let args = env::args();
    let config = Config::from(args);

    let config = Config::new(1, 3, 100, 100.0);

    match mandelbrot_set::run(config) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error: {}", e)
    }
}