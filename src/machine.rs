use crate::array2d::Array2D;
use crate::instructions::{decode, Instruction};
use crate::texture::RGBAImage;
use std::fs::File;
use std::io;
use std::io::Read;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const FONT_START_ADDRESS: usize = 0x50;

const ROM_START_ADDRESS: usize = 0x200;

pub struct Display {
    _pixels: Array2D<bool>,
    _rgba: Vec<u8>,
}

impl Default for Display {
    fn default() -> Self {
        let mut display = Display {
            _pixels: Array2D::new(DISPLAY_HEIGHT, DISPLAY_WIDTH, || false),
            _rgba: vec![0; DISPLAY_HEIGHT * DISPLAY_WIDTH * 4],
        };
        display.update_rgba_from_pixels();
        display
    }
}

impl Display {
    pub fn to_image(&self) -> RGBAImage {
        RGBAImage::new(self._rgba.clone(), self._pixels.cols(), self._pixels.rows())
    }

    pub fn pixels(&self) -> &Array2D<bool> {
        &self._pixels
    }

    pub fn pixel(&self, x: usize, y: usize) -> bool {
        self._pixels[(y, x)]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, v: bool) {
        self._pixels[(y, x)] = v;
        self.update_rgba_from_pixels();
    }

    pub fn clear(&mut self) {
        for i in 0..DISPLAY_HEIGHT {
            for j in 0..DISPLAY_WIDTH {
                self._pixels[(i, j)] = false;
            }
        }
        self.update_rgba_from_pixels();
    }

    fn update_rgba_from_pixels(&mut self) {
        for i in 0..DISPLAY_HEIGHT {
            for j in 0..DISPLAY_WIDTH {
                let color = if self._pixels[(i, j)] { 255 } else { 0 };
                self._rgba[(i * DISPLAY_WIDTH * 4) + j * 4] = color;
                self._rgba[(i * DISPLAY_WIDTH * 4) + j * 4 + 1] = color;
                self._rgba[(i * DISPLAY_WIDTH * 4) + j * 4 + 2] = color;
                self._rgba[(i * DISPLAY_WIDTH * 4) + j * 4 + 3] = color;
            }
        }
    }

    pub fn width(&self) -> usize {
        self._pixels.cols()
    }

    pub fn height(&self) -> usize {
        self._pixels.rows()
    }
}

/// A CHIP8 computer
/// https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
pub struct Machine {
    pub display: Display,
    pub ram: [u8; 4096],
    pub stack: [u8; 100],
    pub program_counter: usize,
    pub index_register: u16,
    pub registers: [u8; 16],
}

impl Default for Machine {
    fn default() -> Self {
        let mut machine = Machine {
            display: Display::default(),
            ram: [0; 4096],
            stack: [0; 100],
            program_counter: 0,
            index_register: 0,
            registers: [0; 16],
        };
        machine.init_font();
        machine
    }
}

impl Machine {
    fn set_flag_register(&mut self, v: u8) {
        self.registers[15] = v;
    }

    fn init_font(&mut self) {
        for (i, &glyph) in FONT.iter().enumerate() {
            self.ram[FONT_START_ADDRESS + i] = glyph;
        }
    }

    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = vec![];
        f.read_to_end(&mut buf)?;

        for (i, v) in buf.iter().enumerate() {
            self.ram[ROM_START_ADDRESS + i] = *v;
        }
        self.program_counter = ROM_START_ADDRESS;
        Ok(())
    }

    pub fn decode_next_instruction(&self) -> Instruction {
        decode(
            ((self.ram[self.program_counter] as u16) << 8)
                | self.ram[self.program_counter + 1] as u16,
        )
    }

    pub fn execute_one(&mut self) {
        let instruction = self.decode_next_instruction();
        self.program_counter += 2;
        match instruction {
            Instruction::Zero => {}
            Instruction::ClearScreen => {
                self.display.clear();
            }
            Instruction::Jump(v) => {
                self.program_counter = v as usize;
            }
            Instruction::SetRegister(reg, val) => {
                self.registers[reg as usize] = val;
            }
            Instruction::AddToRegister(reg, val) => {
                self.registers[reg as usize] += val;
            }
            Instruction::SetIndexRegister(val) => {
                self.index_register = val;
            }
            Instruction::Display(rx, ry, n) => {
                let x = (self.registers[rx as usize] % 64) as usize;
                let y = (self.registers[ry as usize] % 32) as usize;
                self.set_flag_register(0);
                for i in 0..n as usize {
                    if y + i >= self.display.height() {
                        break;
                    }
                    let sprite_data = self.ram[self.index_register as usize + i];
                    for j in 0..8 {
                        if x + j >= self.display.width() {
                            break;
                        }
                        let sprite_val = (sprite_data >> (7 - j)) & 1;
                        if sprite_val == 0 {
                            continue;
                        }
                        if self.display.pixel(x + j, y + i) {
                            self.display.set_pixel(x + j, y + i, false);
                            self.set_flag_register(1);
                        } else {
                            self.display.set_pixel(x + j, y + i, true);
                        }
                    }
                }
            }
            Instruction::Unknown(bytes) => {
                println!("Unknown instruction {:02x?}", bytes)
            }
        }
    }
}

/// https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#font
const FONT: [u8; 80] = [
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
