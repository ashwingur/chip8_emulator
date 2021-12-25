use std::fs;

const RAM: usize = 4096;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;

const FONT: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Processor {
    ram: [u8; RAM],
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    registers: [u8; REGISTER_COUNT],
    stack: [usize; STACK_SIZE],
    i_register: u16,
    delay_register: u8,
    sound_register: u8,
    pc: usize,
    sp: usize,
}

impl Processor {
    fn new() -> Processor {
        let mut ram = [0; RAM];
        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        Processor {
            ram,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            registers: [0; REGISTER_COUNT],
            stack: [0; STACK_SIZE],
            i_register: 0,
            delay_register: 0,
            sound_register: 0,
            pc: 512, // Starts at address 0x200
            sp: 0,
        }
    }

    fn load_rom(&mut self, path: &str) {
        if let Ok(data) = fs::read(path) {
            for (i, item) in data.iter().enumerate() {
                self.ram[i + 512] = *item;
            }
        } else {
            panic!("Unable to load rom");
        }
    }

    fn tick(&mut self) {
        // Get the current opcode
        let opcode = self.get_current_opcode();
        // println!("current opcode is {:X?}", opcode);

        // Run the opcode
        self.run_opcode(opcode);
    }

    fn get_current_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    fn run_opcode(&mut self, opcode: u16) {
        // Split the opcode into nibbles (4bits)
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        // Using variables to make it more readable
        // nnn or addr - A 12-bit value, the lowest 12 bits of the instruction
        // n or nibble - A 4-bit value, the lowest 4 bits of the instruction
        // x - A 4-bit value, the lower 4 bits of the high byte of the instruction
        // y - A 4-bit value, the upper 4 bits of the low byte of the instruction
        // kk or byte - An 8-bit value, the lowest 8 bits of the instruction
        let nnn = (opcode & 0x0FFF) as usize;
        let n = nibbles.3 as u8;
        let x = nibbles.1 as u8;
        let y = nibbles.2 as u8;
        let kk = (opcode & 0x00FF) as u8;

        match nibbles {
            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => (),

            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => (),

            // 0nnn - SYS addr
            (0x0, _, _, _) => (),

            // 1nnn - JP addr
            (0x1, _, _, _) => (),

            // 2nnn - CALL addr
            (0x2, _, _, _) => (),

            // 3xkk - SE Vx, byte
            (0x3, _, _, _) => (),

            // 4xkk - SNE Vx, byte
            (0x4, _, _, _) => (),

            // 5xy0 - SE Vx, Vy
            (0x5, _, _, 0x0) => (),

            // 6xkk - LD Vx, byte
            (0x6, _, _, _) => (),

            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => (),

            // 8xy0 - LD Vx, Vy
            (0x8, _, _, 0x0) => (),

            // 8xy1 - OR Vx, Vy
            (0x8, _, _, 0x1) => (),

            // 8xy2 - AND Vx, Vy
            (0x8, _, _, 0x2) => (),

            // 8xy3 - XOR Vx, Vy
            (0x8, _, _, 0x3) => (),

            // 8xy4 - ADD Vx, Vy
            (0x8, _, _, 0x4) => (),

            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => (),

            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) => (),

            // 8xy7 - SUBN Vx, Vy
            (0x8, _, _, 0x7) => (),

            // 8xyE - SHL Vx {, Vy}
            (0x8, _, _, 0xE) => (),

            // 9xy0 - SNE Vx, Vy
            (0x9, _, _, 0x0) => (),

            // Annn - LD I, addr
            (0xA, _, _, _) => (),

            // Bnnn - JP V0, addr
            (0xB, _, _, _) => (),

            // Cxkk - RND Vx, byte
            (0xC, _, _, _) => (),

            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _, _) => (),

            // Ex9E - SKP Vx
            (0xE, _, 0x9, 0xE) => (),

            // ExA1 - SKNP Vx
            (0xE, _, 0xA, 0x1) => (),

            // Fx07 - LD Vx, DT
            (0xF, _, 0x0, 0x7) => (),

            // Fx0A - LD Vx, K
            (0xF, _, 0x0, 0xA) => (),

            // Fx15 - LD DT, Vx
            (0xF, _, 0x1, 0x5) => (),

            // Fx18 - LD ST, Vx
            (0xF, _, 0x1, 0x8) => (),

            // Fx1E - ADD I, Vx
            (0xF, _, 0x1, 0xE) => (),

            // Fx29 - LD F, Vx
            (0xF, _, 0x2, 0x9) => (),

            // Fx33 - LD B, Vx
            (0xF, _, 0x3, 0x3) => (),

            // Fx55 - LD [I], Vx
            (0xF, _, 0x5, 0x5) => (),

            // Fx65 - LD Vx, [I]
            (0xF, _, 0x6, 0x5) => (),

            _ => (),
        }
    }

    /*  00E0 - CLS
        Clear the display.
    */
    fn op_00e0(&mut self) {
        for row in 0..DISPLAY_HEIGHT {
            for pixel in 0..DISPLAY_WIDTH {
                self.display[row][pixel] = 0;
            }
        }
    }

    /*  00EE - RET
        Return from a subroutine.

        The interpreter sets the program counter to the address at the top of the stack,
        then subtracts 1 from the stack pointer.
    */
    fn op_00ee(&mut self) {
        self.pc = self.stack[self.sp];
        self.sp -= 1;
    }

    /*  1nnn - JP addr
        Jump to location nnn.

        The interpreter sets the program counter to nnn.
    */
    fn op_1nnn(&mut self, nnn: usize) {
        self.pc = nnn;
    }

    fn print_ram(&self, condensed: bool) {
        println!("RAM:");
        for i in 512..RAM {
            if condensed {
                print!("{:X?}, ", self.ram[i]);
            } else {
                print!("{}: {:X?}", i, self.ram[i]);
            }
        }
        println!();
    }
}

fn main() {
    let rom = "roms/PONG";

    let mut processor = Processor::new();
    processor.load_rom(rom);
    processor.print_ram(true);
    processor.tick();
}
