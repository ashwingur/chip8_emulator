/*
    Chip-8 technical reference - http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
*/

use std::fs;
extern crate rand;
use crate::display::GameCanvas;
use rand::Rng;
use std::{thread, time};

const RAM: usize = 4096;
pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const INSTRUCTION_SIZE: usize = 2;
const FRAME_DURATION_MILLIS: u64 = 30; // Clock speed of CHIP-8 is usually 500Hz
pub const KEYBOARD_SIZE: usize = 16;

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

#[cfg(feature = "debug_mode")]
macro_rules! debug {
    ($x:expr) => {
        println!($x)
    };
}

#[cfg(not(feature = "debug_mode"))]
macro_rules! debug {
    ($x:expr) => {
        std::convert::identity($x)
    };
}

pub struct Processor {
    ram: [u8; RAM],
    display: [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
    v: [u8; REGISTER_COUNT],
    stack: [usize; STACK_SIZE],
    i_register: usize,
    delay_register: u8,
    sound_register: u8,
    pc: usize,
    sp: usize,
    keyboard_presses: [bool; KEYBOARD_SIZE],
}

impl Processor {
    pub fn new() -> Processor {
        // Load in the default sprites
        let mut ram = [0; RAM];
        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        Processor {
            ram,
            display: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            v: [0; REGISTER_COUNT],
            stack: [0; STACK_SIZE],
            i_register: 0,
            delay_register: 0,
            sound_register: 0,
            pc: 512, // Starts at address 0x200
            sp: 0,
            keyboard_presses: [false; KEYBOARD_SIZE],
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        if let Ok(data) = fs::read(path) {
            for (i, item) in data.iter().enumerate() {
                self.ram[i + 512] = *item;
            }
        } else {
            panic!("Unable to load rom");
        }
    }

    pub fn start(&mut self) {
        let mut game_canvas = GameCanvas::new();
        loop {
            if let Some(input) = game_canvas.read_keyboard_inputs() {
            } else {
                break;
            }
            self.tick();
            game_canvas.draw_frame(&self.display);
            thread::sleep(time::Duration::from_millis(FRAME_DURATION_MILLIS));
        }
    }

    pub fn tick(&mut self) {
        // Get the current opcode
        let opcode = self.get_current_opcode();

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
        let n = nibbles.3 as usize;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let kk = (opcode & 0x00FF) as u8;

        match nibbles {
            // 00EE - RET
            (0x0, 0x0, 0xE, 0xE) => self.op_00ee(),

            // 00E0 - CLS
            (0x0, 0x0, 0xE, 0x0) => self.op_00e0(),

            // 0nnn - SYS addr
            (0x0, _, _, _) => (),

            // 1nnn - JP addr
            (0x1, _, _, _) => self.op_1nnn(nnn),

            // 2nnn - CALL addr
            (0x2, _, _, _) => self.op_2nnn(nnn),

            // 3xkk - SE Vx, byte
            (0x3, _, _, _) => self.op_3xkk(x, kk),

            // 4xkk - SNE Vx, byte
            (0x4, _, _, _) => self.op_4xkk(x, kk),

            // 5xy0 - SE Vx, Vy
            (0x5, _, _, 0x0) => self.op_5xy0(x, y),

            // 6xkk - LD Vx, byte
            (0x6, _, _, _) => self.op_6xkk(x, kk),

            // 7xkk - ADD Vx, byte
            (0x7, _, _, _) => self.op_7xkk(x, kk),

            // 8xy0 - LD Vx, Vy
            (0x8, _, _, 0x0) => self.op_8xy0(x, y),

            // 8xy1 - OR Vx, Vy
            (0x8, _, _, 0x1) => self.op_8xy1(x, y),

            // 8xy2 - AND Vx, Vy
            (0x8, _, _, 0x2) => self.op_8xy2(x, y),

            // 8xy3 - XOR Vx, Vy
            (0x8, _, _, 0x3) => self.op_8xy3(x, y),

            // 8xy4 - ADD Vx, Vy
            (0x8, _, _, 0x4) => self.op_8xy4(x, y),

            // 8xy5 - SUB Vx, Vy
            (0x8, _, _, 0x5) => self.op_8xy5(x, y),

            // 8xy6 - SHR Vx {, Vy}
            (0x8, _, _, 0x6) => self.op_8xy6(x),

            // 8xy7 - SUBN Vx, Vy
            (0x8, _, _, 0x7) => self.op_8xy7(x, y),

            // 8xyE - SHL Vx {, Vy}
            (0x8, _, _, 0xE) => self.op_8xye(x),

            // 9xy0 - SNE Vx, Vy
            (0x9, _, _, 0x0) => self.op_9xy0(x, y),

            // Annn - LD I, addr
            (0xA, _, _, _) => self.op_annn(nnn),

            // Bnnn - JP V0, addr
            (0xB, _, _, _) => self.op_bnnn(nnn),

            // Cxkk - RND Vx, byte
            (0xC, _, _, _) => self.op_cxkk(x, kk),

            // Dxyn - DRW Vx, Vy, nibble
            (0xD, _, _, _) => self.op_dxyn(x, y, n),

            // Ex9E - SKP Vx
            (0xE, _, 0x9, 0xE) => self.op_ex9e(x),

            // ExA1 - SKNP Vx
            (0xE, _, 0xA, 0x1) => self.op_exa1(x),

            // Fx07 - LD Vx, DT
            (0xF, _, 0x0, 0x7) => self.op_fx07(x),

            // Fx0A - LD Vx, K
            (0xF, _, 0x0, 0xA) => self.op_fx0a(x),

            // Fx15 - LD DT, Vx
            (0xF, _, 0x1, 0x5) => self.op_fx15(x),

            // Fx18 - LD ST, Vx
            (0xF, _, 0x1, 0x8) => self.op_fx18(x),

            // Fx1E - ADD I, Vx
            (0xF, _, 0x1, 0xE) => self.op_fx1e(x),

            // Fx29 - LD F, Vx
            (0xF, _, 0x2, 0x9) => self.op_fx29(x),

            // Fx33 - LD B, Vx
            (0xF, _, 0x3, 0x3) => self.op_fx33(x),

            // Fx55 - LD [I], Vx
            (0xF, _, 0x5, 0x5) => self.op_fx55(x),

            // Fx65 - LD Vx, [I]
            (0xF, _, 0x6, 0x5) => self.op_fx65(x),

            _ => (),
        }
    }

    /*  00E0 - CLS
        Clear the display.
    */
    fn op_00e0(&mut self) {
        debug!("CLS");
        for row in 0..DISPLAY_HEIGHT {
            for pixel in 0..DISPLAY_WIDTH {
                self.display[row][pixel] = 0;
            }
        }
        self.pc += INSTRUCTION_SIZE;
    }

    /*  00EE - RET
        Return from a subroutine.

        The interpreter sets the program counter to the address at the top of the stack,
        then subtracts 1 from the stack pointer.
    */
    fn op_00ee(&mut self) {
        debug!("RET");
        self.pc = self.stack[self.sp];
        self.sp -= 1;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  1nnn - JP addr
        Jump to location nnn.

        The interpreter sets the program counter to nnn.
    */
    fn op_1nnn(&mut self, nnn: usize) {
        debug!("1nnn");
        self.pc = nnn;
    }

    /*  2nnn - CALL addr
        Call subroutine at nnn.

        The interpreter increments the stack pointer, then puts the current
        PC on the top of the stack. The PC is then set.
    */
    fn op_2nnn(&mut self, nnn: usize) {
        debug!("2nnn");
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = nnn;
    }

    /*  3xkk - SE Vx, byte
        Skip next instruction if Vx = kk.

        The interpreter compares register Vx to kk, and if they
        are equal, increments the program counter by 2.
    */
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        debug!("3xkk");
        self.pc += if self.v[x] == kk {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        }
    }

    /*  4xkk - SNE Vx, byte
        Skip next instruction if Vx != kk.

        The interpreter compares register Vx to kk, and if they are
        not equal, increments the program counter by 2.
    */
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        debug!("4xkk");
        self.pc += if self.v[x] != kk {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        }
    }

    /*  5xy0 - SE Vx, Vy
        Skip next instruction if Vx = Vy.

        The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    */
    fn op_5xy0(&mut self, x: usize, y: usize) {
        debug!("5xy0");
        self.pc += if self.v[x] == self.v[y] {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        }
    }

    /*  6xkk - LD Vx, byte
        Set Vx = kk.

        The interpreter puts the value kk into register Vx.
    */
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        debug!("6xkk");
        self.v[x] = kk;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  7xkk - ADD Vx, byte
        Set Vx = Vx + kk.

        Adds the value kk to the value of register Vx,
        then stores the result in Vx.
    */
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        debug!("7xkk");
        self.v[x] = self.v[x].wrapping_add(kk);
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy0 - LD Vx, Vy
        Set Vx = Vy.

        Stores the value of register Vy in register Vx.
    */
    fn op_8xy0(&mut self, x: usize, y: usize) {
        debug!("8xy0");
        self.v[x] = self.v[y];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy1 - OR Vx, Vy
        Set Vx = Vx OR Vy.

        Performs a bitwise OR on the values of Vx and Vy,
        then stores the result in Vx.
    */
    fn op_8xy1(&mut self, x: usize, y: usize) {
        debug!("8xy1");
        self.v[x] |= self.v[y];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy2 - AND Vx, Vy
        Set Vx = Vx AND Vy.

        Performs a bitwise AND on the values of Vx and Vy,
        then stores the result in Vx.
    */
    fn op_8xy2(&mut self, x: usize, y: usize) {
        debug!("8xy2");
        self.v[x] &= self.v[y];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy3 - XOR Vx, Vy
        Set Vx = Vx XOR Vy.

        Performs a bitwise exclusive OR on the values of Vx and Vy,
        then stores the result in Vx.
    */
    fn op_8xy3(&mut self, x: usize, y: usize) {
        debug!("8xy3");
        self.v[x] ^= self.v[y];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy4 - ADD Vx, Vy
        Set Vx = Vx + Vy, set VF = carry.

        The values of Vx and Vy are added together. If the result is greater
        than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
        Only the lowest 8 bits of the result are kept, and stored in Vx.
    */
    fn op_8xy4(&mut self, x: usize, y: usize) {
        debug!("8xy4");
        self.v[x] = match self.v[x].overflowing_add(self.v[y]) {
            (sum, true) => {
                // Overflow occurred
                self.v[0xF] = 1;
                sum
            }
            (sum, false) => {
                self.v[0xF] = 0;
                sum
            }
        };
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy5 - SUB Vx, Vy
        Set Vx = Vx - Vy, set VF = NOT borrow.

        If Vx > Vy, then VF is set to 1, otherwise 0.
        Then Vy is subtracted from Vx, and the results stored in Vx.
    */
    fn op_8xy5(&mut self, x: usize, y: usize) {
        debug!("8xy5");
        self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy6 - SHR Vx {, Vy}
        Set Vx = Vx SHR 1.

        If the least-significant bit of Vx is 1, then VF is set to 1,
        otherwise 0. Then Vx is divided by 2.
    */
    fn op_8xy6(&mut self, x: usize) {
        debug!("8xy6");
        self.v[0xF] = self.v[x] & 1u8;
        self.v[x] >>= 1;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xy7 - SUBN Vx, Vy
        Set Vx = Vy - Vx, set VF = NOT borrow.

        If Vy > Vx, then VF is set to 1, otherwise 0.
        Then Vx is subtracted from Vy, and the results stored in Vx.
    */
    fn op_8xy7(&mut self, x: usize, y: usize) {
        debug!("8xy7");
        self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        self.pc += INSTRUCTION_SIZE;
    }

    /*  8xyE - SHL Vx {, Vy}
        Set Vx = Vx SHL 1.

        If the most-significant bit of Vx is 1, then VF is set to 1,
        otherwise to 0. Then Vx is multiplied by 2.
    */
    fn op_8xye(&mut self, x: usize) {
        debug!("8xye");
        self.v[0xF] = self.v[x] & 1u8;
        self.v[x] <<= 1;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  9xy0 - SNE Vx, Vy
        Skip next instruction if Vx != Vy.

        The values of Vx and Vy are compared, and if they
        are not equal, the program counter is increased by 2.
    */
    fn op_9xy0(&mut self, x: usize, y: usize) {
        debug!("9xy0");
        self.pc += if self.v[x] != self.v[y] {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        };
    }

    /*  Annn - LD I, addr
        Set I = nnn.

        The value of register I is set to nnn.
    */
    fn op_annn(&mut self, nnn: usize) {
        debug!("annn");
        self.i_register = nnn;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Bnnn - JP V0, addr
        Jump to location nnn + V0.

        The program counter is set to nnn plus the value of V0.
    */
    fn op_bnnn(&mut self, nnn: usize) {
        debug!("bnnn");
        self.pc = nnn + self.v[0x0] as usize;
    }

    /*  Cxkk - RND Vx, byte
        Set Vx = random byte AND kk.

        The interpreter generates a random number from 0 to 255,
        which is then ANDed with the value kk. The results are stored in Vx.
    */
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        debug!("cxkk");
        let random_byte: u8 = rand::thread_rng().gen();
        self.v[x] = kk & random_byte;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Dxyn - DRW Vx, Vy, nibble
        Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.

        The interpreter reads n bytes from memory, starting at the address stored in I.
        These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
        Sprites are XORed onto the existing screen. If this causes any pixels to be erased,
        VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of
        it is outside the coordinates of the display, it wraps around to the opposite
        side of the screen.
    */
    fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        debug!("dxyn");
        self.v[0xF] = 0;
        for row in 0..n {
            let y_index = (self.v[y] as usize + row) % DISPLAY_HEIGHT;
            for col in 0..8 {
                // XOR onto the screen, if a pixel became 0 set VF to 1
                // Wrapping around the screen if required
                let x_index = (self.v[x] as usize + col) % DISPLAY_WIDTH;
                let pixel = (self.ram[self.i_register + row] >> (7 - col)) & 1;
                self.display[y_index][x_index] ^= pixel;
                if self.display[y_index][x_index] == 0 {
                    self.v[0xF] = 1;
                }
            }
        }
        // self.print_ascii_display();
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Ex9E - SKP Vx
        Skip next instruction if key with the value of Vx is pressed.

        Checks the keyboard, and if the key corresponding to the value
        of Vx is currently in the down position, PC is increased by 2.
    */
    fn op_ex9e(&mut self, x: usize) {
        debug!("ex9e");
        self.pc += if self.keyboard_presses[self.v[x] as usize] == true {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        };
    }

    /*  ExA1 - SKNP Vx
        Skip next instruction if key with the value of Vx is not pressed.

        Checks the keyboard, and if the key corresponding to the value of
        Vx is currently in the up position, PC is increased by 2.
    */
    fn op_exa1(&mut self, x: usize) {
        debug!("exa1");
        self.pc += if self.keyboard_presses[self.v[x] as usize] == false {
            2 * INSTRUCTION_SIZE
        } else {
            INSTRUCTION_SIZE
        };
    }

    /*  Fx07 - LD Vx, DT
        Set Vx = delay timer value.

        The value of DT is placed into Vx.
    */
    fn op_fx07(&mut self, x: usize) {
        debug!("fx07");
        // TODO
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx0A - LD Vx, K
        Wait for a key press, store the value of the key in Vx.

        All execution stops until a key is pressed,
        then the value of that key is stored in Vx.
    */
    fn op_fx0a(&mut self, x: usize) {
        debug!("fx0a");
        // TODO
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx15 - LD DT, Vx
        Set delay timer = Vx.

        DT is set equal to the value of Vx.
    */
    fn op_fx15(&mut self, x: usize) {
        debug!("fx15");
        self.delay_register = self.v[x];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx18 - LD ST, Vx
        Set sound timer = Vx.

        ST is set equal to the value of Vx.
    */
    fn op_fx18(&mut self, x: usize) {
        debug!("fx18");
        self.sound_register = self.v[x];
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx1E - ADD I, Vx
        Set I = I + Vx.

        The values of I and Vx are added, and the results are stored in I.
    */
    fn op_fx1e(&mut self, x: usize) {
        debug!("fx1e");
        self.i_register += self.v[x] as usize;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx29 - LD F, Vx
        Set I = location of sprite for digit Vx.

        The value of I is set to the location for
        the hexadecimal sprite corresponding to the value of Vx.
    */
    fn op_fx29(&mut self, x: usize) {
        debug!("fx29");
        self.i_register = (self.v[x] * 5) as usize;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx33 - LD B, Vx
        Store BCD representation of Vx in memory locations I, I+1, and I+2.

        The interpreter takes the decimal value of Vx, and places the hundreds
        digit in memory at location in I, the tens digit at location I+1, and
        the ones digit at location I+2.
    */
    fn op_fx33(&mut self, x: usize) {
        debug!("fx33");
        self.ram[self.i_register] = self.v[x] / 100;
        self.ram[self.i_register + 1] = (self.v[x] % 100) / 10;
        self.ram[self.i_register + 2] = self.v[x] % 10;
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx55 - LD [I], Vx
        Store registers V0 through Vx in memory starting at location I.

        The interpreter copies the values of registers V0 through Vx into
        memory, starting at the address in I.
    */
    fn op_fx55(&mut self, x: usize) {
        debug!("fx55");
        for i in 0..(x + 1) {
            self.ram[self.i_register + i] = self.v[i];
        }
        self.pc += INSTRUCTION_SIZE;
    }

    /*  Fx65 - LD Vx, [I]
        Read registers V0 through Vx from memory starting at location I.

        The interpreter reads values from memory starting at location I
        into registers V0 through Vx.
    */
    fn op_fx65(&mut self, x: usize) {
        debug!("fx65");
        for i in 0..(x + 1) {
            self.v[i] = self.ram[self.i_register + i];
        }
        self.pc += INSTRUCTION_SIZE;
    }

    pub fn print_ram(&self, condensed: bool) {
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

    pub fn print_ascii_display(&self) {
        for row in self.display {
            for cell in row {
                if cell == 1 {
                    print!("█");
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}
