use mandelbrot_set::Config;

fn main() {
    let config = Config::new(1, 4, 1000, 100.0);

    match mandelbrot_set::run(config) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error: {}", e)
    }
}
