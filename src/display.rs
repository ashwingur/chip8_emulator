extern crate sdl2;
use crate::DISPLAY_HEIGHT;
use crate::DISPLAY_WIDTH;
use crate::KEYBOARD_SIZE;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use sdl2::Sdl;

const PIXEL_SCALE: u32 = 12;

pub struct GameCanvas {
    canvas: Canvas<Window>,
    sdl_context: Sdl,
    event_pump: EventPump,
}

impl GameCanvas {
    pub fn new() -> GameCanvas {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "rust-sdl2 demo",
                DISPLAY_WIDTH as u32 * PIXEL_SCALE,
                DISPLAY_HEIGHT as u32 * PIXEL_SCALE,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        GameCanvas {
            canvas,
            sdl_context,
            event_pump: event_pump,
        }
    }

    pub fn read_keyboard_inputs(&mut self) -> Option<[bool; KEYBOARD_SIZE]> {
        self.event_pump.pump_events();
        if self
            .event_pump
            .keyboard_state()
            .is_scancode_pressed(Scancode::Escape)
        {
            return None;
        }

        let mut keys = [false; KEYBOARD_SIZE];

        // The 4x4 keyboard will be mapped to the following keys
        /*
            1 2 3 C           1 2 3 4
            4 5 6 D  --->     q w e r
            7 8 9 E           a s d f
            A 0 B F           z x c v
        */
        for s in self.event_pump.keyboard_state().pressed_scancodes() {
            let code = match s {
                Scancode::X => Some(0x0),
                Scancode::Num1 => Some(0x1),
                Scancode::Num2 => Some(0x2),
                Scancode::Num3 => Some(0x3),
                Scancode::Q => Some(0x4),
                Scancode::W => Some(0x5),
                Scancode::E => Some(0x6),
                Scancode::A => Some(0x7),
                Scancode::S => Some(0x8),
                Scancode::D => Some(0x9),
                Scancode::Z => Some(0xA),
                Scancode::C => Some(0xB),
                Scancode::Num4 => Some(0xC),
                Scancode::R => Some(0xD),
                Scancode::F => Some(0xE),
                Scancode::V => Some(0xF),
                Scancode::Escape => return None,
                _ => None,
            };

            if let Some(i) = code {
                keys[i] = true;
            }
        }

        Some(keys)
    }

    pub fn draw_frame(&mut self, display: &[[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT]) {
        // Set the whole background to black
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                if display[row][col] == 1 {
                    let _ = self.canvas.fill_rect(Rect::new(
                        (col as u32 * PIXEL_SCALE) as i32,
                        (row as u32 * PIXEL_SCALE) as i32,
                        PIXEL_SCALE,
                        PIXEL_SCALE,
                    ));
                }
            }
        }
        self.canvas.present();
    }
}
