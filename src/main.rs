use mandelbrot_set::Config;
use std::{env, process};
use std::alloc::System;

fn main() {
    let args = env::args();
    let config = Config::from(args);

    let config = config.unwrap_or_else(|err| {
        eprintln!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    //let config = Config::new(1, 4, 1000, 100.0);

    match mandelbrot_set::run(config) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error: {}", e)
    }
}