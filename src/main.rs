mod processor;
use processor::*;
mod display;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run <rom>");
        return;
    }

    let mut processor = Processor::new();
    processor.load_rom(&format!("roms/{}", &args[1]));
    processor.start();
}
