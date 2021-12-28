mod processor;
use processor::*;
mod display;

fn main() {
    let rom = "roms/INVADERS";

    let mut processor = Processor::new();
    processor.load_rom(rom);
    // processor.print_ram(true);
    // processor.tick();
    processor.start();
}
