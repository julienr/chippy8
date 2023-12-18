use crate::array2d::Array2D;
use std::fs::File;
use std::io;
use std::io::Read;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

const FONT_START_ADDRESS: usize = 0x50;

const ROM_START_ADDRESS: usize = 0x200;

/// A CHIP8 computer
/// https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
pub struct Machine {
    pub display: Array2D<bool>,
    pub ram: [u8; 4096],
    pub stack: [u8; 100],
    pub program_counter: usize,
    pub index_register: u16,
    pub registers: [u8; 16],
}

impl Default for Machine {
    fn default() -> Self {
        let mut machine = Machine {
            display: Array2D::new(DISPLAY_HEIGHT, DISPLAY_WIDTH, || false),
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
    fn init_font(&mut self) {
        for i in 0..FONT.len() {
            self.ram[FONT_START_ADDRESS + i] = FONT[i];
        }
    }

    pub fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = vec![0];
        f.read_to_end(&mut buf)?;

        for (i, v) in buf.iter().enumerate() {
            self.ram[ROM_START_ADDRESS + i] = *v;
        }
        Ok(())
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
