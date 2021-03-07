use mandelbrot_set::Config;
use std::{env, process};

fn main() {
    let args = env::args();
    let config = Config::from(args);

    let config = config.unwrap_or_else(|err| {
        eprintln!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    match mandelbrot_set::run(config) {
        Ok(_) => (),
        Err(e) => println!("Error: {}", e)
    }
}

