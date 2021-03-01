use mandelbrot_set::Config;

fn main() {
    let config = Config::new(1, 3, 100, 100.0);

    match mandelbrot_set::main(config) {
        Ok(s) => println!("{}", s),
        Err(e) => println!("Error: {}", e)
    }
}
