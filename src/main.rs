mod processor;
use processor::*;

fn main() {
    let rom = "roms/MAZE";

    let mut processor = Processor::new();
    processor.load_rom(rom);
    // processor.print_ram(true);
    // processor.tick();
    processor.start();
}
