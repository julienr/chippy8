use crate::array2d::Array2D;
use crate::instructions::{decode, Instruction};
use crate::texture::RGBAImage;
use rand::Rng;
use std::fs::File;
use std::io;
use std::io::Read;
use std::num::Wrapping;

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
    pub stack: [u16; 100],
    stack_index: usize,
    pub program_counter: usize,
    pub index_register: u16,
    pub registers: [u8; 16],
    pub key_pressed: [bool; 16],
}

impl Default for Machine {
    fn default() -> Self {
        let mut machine = Machine {
            display: Display::default(),
            ram: [0; 4096],
            stack: [0; 100],
            stack_index: 0,
            program_counter: 0,
            index_register: 0,
            registers: [0; 16],
            key_pressed: [false; 16],
        };
        machine.init_font();
        machine
    }
}

impl Machine {
    fn set_flag_register(&mut self, v: u8) {
        self.registers[15] = v;
    }

    pub fn flag_register(&self) -> u8 {
        self.registers[15]
    }

    fn push_stack(&mut self, v: u16) {
        if self.stack_index > self.stack.len() {
            // TODO: Warning logs / flag to display in UI ?
            println!("maximum stack depth exceeded ({:?})", self.stack.len());
            self.stack_index = self.stack.len() - 1;
        }
        self.stack[self.stack_index] = v;
        self.stack_index += 1;
    }

    fn pop_stack(&mut self) -> u16 {
        if self.stack_index < 1 {
            // TODO: Warning logs / flag to display in UI ?
            println!("trying to pop from empty stack");
            self.stack_index = 1;
        }
        self.stack_index -= 1;
        self.stack[self.stack_index]
    }

    fn init_font(&mut self) {
        for (i, &glyph) in FONT.iter().enumerate() {
            self.ram[FONT_START_ADDRESS + i] = glyph;
        }
    }

    /// Load from bytes
    pub fn load_rom_from_bytes(&mut self, data: &[u8]) {
        for (i, v) in data.iter().enumerate() {
            self.ram[ROM_START_ADDRESS + i] = *v;
        }
        self.program_counter = ROM_START_ADDRESS;
    }

    /// Load from a ROM file
    pub fn load_rom_from_file(&mut self, filename: &str) -> io::Result<()> {
        let mut f = File::open(filename)?;
        let mut buf: Vec<u8> = vec![];
        f.read_to_end(&mut buf)?;

        self.load_rom_from_bytes(&buf);
        Ok(())
    }

    /// A helper to load from 2 bytes at a time, which make it easy to write
    /// instructions in hexa; mostly for testing
    pub fn load_rom_from_instrhex(&mut self, data: &[u16]) {
        let mut bytes: Vec<u8> = vec![];
        for v in data.iter() {
            bytes.push(((v & 0xFF00) >> 8) as u8);
            bytes.push((v & 0x00FF) as u8);
        }
        self.load_rom_from_bytes(&bytes);
    }

    pub fn decode_next_instruction(&self) -> Instruction {
        decode(
            ((self.ram[self.program_counter] as u16) << 8)
                | self.ram[self.program_counter + 1] as u16,
            &format!("pc={:#02x}", self.program_counter),
        )
    }

    fn _execute_subtract(&mut self, rx: u8, v1: u8, v2: u8) {
        // TODO: Not sure if > or >=, tobiasvl guide is not crystal clear
        if v1 >= v2 {
            self.set_flag_register(1);
        } else {
            self.set_flag_register(0);
        }
        self.registers[rx as usize] = (Wrapping(v1) - Wrapping(v2)).0;
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
            Instruction::SetRegToVal(reg, val) => {
                self.registers[reg as usize] = val;
            }
            Instruction::AddValToReg(reg, val) => {
                let v = Wrapping(self.registers[reg as usize]) + Wrapping(val);
                self.registers[reg as usize] = v.0;
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
            Instruction::Subroutine(v) => {
                println!("calling subroutine");
                self.push_stack(self.program_counter as u16);
                self.program_counter = v as usize;
            }
            Instruction::Return => {
                self.program_counter = self.pop_stack() as usize;
            }
            Instruction::Unknown(bytes, location_int) => {
                println!("Unknown instruction {:#02x?} at {:?}", bytes, location_int)
            }
            Instruction::SkipIfEqualRegVal(reg, val) => {
                if self.registers[reg as usize] == val {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipIfNotEqualRegVal(reg, val) => {
                if self.registers[reg as usize] != val {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipIfEqualRegReg(reg1, reg2) => {
                if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::SkipIfNotEqualRegReg(reg1, reg2) => {
                if self.registers[reg1 as usize] != self.registers[reg2 as usize] {
                    self.program_counter += 2;
                }
            }
            Instruction::Set(rx, ry) => {
                self.registers[rx as usize] = self.registers[ry as usize];
            }
            Instruction::Or(rx, ry) => {
                self.registers[rx as usize] |= self.registers[ry as usize];
            }
            Instruction::And(rx, ry) => {
                self.registers[rx as usize] &= self.registers[ry as usize];
            }
            Instruction::Xor(rx, ry) => {
                self.registers[rx as usize] ^= self.registers[ry as usize];
            }
            Instruction::Add(rx, ry) => {
                let v1 = self.registers[rx as usize];
                let v2 = self.registers[ry as usize];
                if v1 as usize + v2 as usize > 255 {
                    self.set_flag_register(1);
                } else {
                    self.set_flag_register(0);
                }
                self.registers[rx as usize] = (Wrapping(v1) + Wrapping(v2)).0;
            }
            Instruction::SubtractXY(rx, ry) => {
                self._execute_subtract(
                    rx,
                    self.registers[rx as usize],
                    self.registers[ry as usize],
                );
            }
            Instruction::SubtractYX(rx, ry) => {
                self._execute_subtract(
                    rx,
                    self.registers[ry as usize],
                    self.registers[rx as usize],
                );
            }
            Instruction::ShiftLeft(rx, _) => {
                // TODO: Need a feature flag here because this is ambiguous
                let v = self.registers[rx as usize];
                self.set_flag_register((v >> 7) & 1);
                self.registers[rx as usize] = v << 1;
            }
            Instruction::ShiftRight(rx, _) => {
                // TODO: Need a feature flag here because this is ambiguous
                let v = self.registers[rx as usize];
                self.set_flag_register(v & 1);
                self.registers[rx as usize] = v >> 1;
            }
            Instruction::JumpWithOffset(offset) => {
                // TODO: Ambiguous one, may want a feature flag
                self.program_counter = offset as usize + self.registers[0] as usize;
            }
            Instruction::Random(rx, v) => {
                let mut rng = rand::thread_rng();
                let n1 = rng.gen::<u8>();
                self.registers[rx as usize] = n1 & v;
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

#[cfg(test)]
mod tests {
    use super::*;

    impl Machine {
        fn from_instrhex(data: &[u16]) -> Machine {
            let mut machine = Machine::default();
            machine.load_rom_from_instrhex(data);
            machine
        }
    }

    #[test]
    fn test_instr_clear_screen() {
        let mut machine = Machine::from_instrhex(&[0x00E0]);
        machine.display.set_pixel(5, 7, true);
        machine.execute_one();
        assert!(!machine.display.pixel(5, 7));
    }

    #[test]
    fn test_instr_jump() {
        let mut machine = Machine::from_instrhex(&[0x1ABC]);
        assert_eq!(machine.program_counter, 0x200);
        machine.execute_one();
        assert_eq!(machine.program_counter, 0xABC);
    }

    #[test]
    fn test_instr_set_reg_to_val() {
        let mut machine = Machine::from_instrhex(&[0x6CDE]);
        machine.execute_one();
        assert_eq!(machine.registers[0xC], 0xDE);
    }

    #[test]
    fn test_instr_add_val_to_reg() {
        let mut machine = Machine::from_instrhex(&[0x7032]);
        // Test that overflow cycles and doesn't set the VF register
        // https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#7xnn-add
        machine.registers[0] = 0xFF;
        machine.execute_one();
        assert_eq!(machine.registers[0], 0x31);
        assert_eq!(machine.flag_register(), 0);
    }

    #[test]
    fn test_instr_set_index_register() {
        let mut machine = Machine::from_instrhex(&[0xABCD]);
        machine.execute_one();
        assert_eq!(machine.index_register, 0xBCD);
    }

    #[test]
    fn test_instr_display() {
        let mut machine = Machine::from_instrhex(&[0xD123, 0xD123]);
        machine.registers[1] = 10;
        machine.registers[2] = 5;
        machine.index_register = 0x345;
        machine.ram[0x345] = 0b11110000;
        machine.ram[0x345 + 1] = 0b00001111;
        machine.ram[0x345 + 2] = 0b001111000;
        // This line shouldn't be displayed (N=3 in the instruction above)
        machine.ram[0x345 + 3] = 0b10100000;

        assert_eq!(machine.display.pixels().count_value(true), 0);

        // ==== First execution should show the sprite
        machine.execute_one();
        // First row
        assert!(machine.display.pixel(10, 5));
        assert!(machine.display.pixel(11, 5));
        assert!(!machine.display.pixel(14, 5));
        // Second row
        assert!(!machine.display.pixel(13, 6));
        assert!(machine.display.pixel(14, 6));
        // Third row
        assert!(!machine.display.pixel(10, 7));
        assert!(machine.display.pixel(14, 7));
        // Total white pixels
        assert_eq!(machine.display.pixels().count_value(true), 12);

        // ==== Executing a second time should erase it
        machine.execute_one();
        assert_eq!(machine.display.pixels().count_value(true), 0);
    }

    #[test]
    fn test_instr_subroutine() {
        let mut machine = Machine::from_instrhex(&[
            0x2000 + ROM_START_ADDRESS as u16 + 6, // calls subroutine starting two instruction below
            0x6001,                                // set v0 to '1'
            0x1000 + ROM_START_ADDRESS as u16 + 4, // infinite loop to self
            // subroutine:
            0x6002, // set v0 to '2'
            0x00EE, // return
        ]);
        for _ in 0..5 {
            machine.execute_one();
        }
        assert_eq!(machine.registers[0], 1);
        // We should have poped from the stack
        assert_eq!(machine.stack_index, 0);
    }

    #[test]
    fn test_instr_skip_if_equal_reg_val() {
        let mut machine = Machine::from_instrhex(&[
            0x3210, // skip if equal => will skip next
            0x1FFF, // jump to invalid address, this should be skipped
            0x3211, // skip if equal => this shouldn't skip
            0x61FF, // set a register
        ]);
        machine.registers[2] = 0x10;
        for _ in 0..4 {
            machine.execute_one();
        }
        assert_eq!(machine.registers[1], 0xFF);
    }

    #[test]
    fn test_instr_skip_if_not_equal_reg_val() {
        let mut machine = Machine::from_instrhex(&[
            0x4211, // skip if not equal => will skip next
            0x1FFF, // jump to invalid address, this should be skipped
            0x4210, // skip if not equal => this shouldn't skip
            0x61FF, // set a register
        ]);
        machine.registers[2] = 0x10;
        for _ in 0..4 {
            machine.execute_one();
        }
        assert_eq!(machine.registers[1], 0xFF);
    }

    #[test]
    fn test_instr_skip_if_equal_reg_reg() {
        let mut machine = Machine::from_instrhex(&[
            0x5120, // skip if equal => should skip
            0x1FFF, // jump to invalid address, this should be skipped
            0x5130, // skip if equal => this shouldn't skip
            0x60FF, // set a register
        ]);
        machine.registers[1] = 0x11;
        machine.registers[2] = 0x11;
        machine.registers[3] = 0x0;
        for _ in 0..4 {
            machine.execute_one();
        }
        assert_eq!(machine.registers[0], 0xFF);
    }

    #[test]
    fn test_instr_skip_if_not_equal_reg_reg() {
        let mut machine = Machine::from_instrhex(&[
            0x9130, // skip if not equal => should skip
            0x1FFF, // jump to invalid address, this should be skipped
            0x9120, // skip if not equal => this shouldn't skip
            0x60FF, // set a register
        ]);
        machine.registers[1] = 0x11;
        machine.registers[2] = 0x11;
        machine.registers[3] = 0x0;
        for _ in 0..4 {
            machine.execute_one();
        }
        assert_eq!(machine.registers[0], 0xFF);
    }

    #[test]
    fn test_instr_set() {
        let mut machine = Machine::from_instrhex(&[0x8120]);
        machine.registers[1] = 0x42;
        machine.registers[2] = 0x43;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0x43);
        assert_eq!(machine.registers[2], 0x43);
    }

    #[test]
    fn test_instr_or() {
        let mut machine = Machine::from_instrhex(&[0x8121]);
        machine.registers[1] = 0b0101;
        machine.registers[2] = 0b0011;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0b0111);
    }

    #[test]
    fn test_instr_and() {
        let mut machine = Machine::from_instrhex(&[0x8122]);
        machine.registers[1] = 0b0101;
        machine.registers[2] = 0b0011;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0b0001);
    }

    #[test]
    fn test_instr_xor() {
        let mut machine = Machine::from_instrhex(&[0x8123]);
        machine.registers[1] = 0b0101;
        machine.registers[2] = 0b0011;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0b0110);
    }

    #[test]
    fn test_instr_add() {
        let mut machine = Machine::from_instrhex(&[0x8124, 0x8124]);
        machine.registers[1] = 250;
        machine.registers[2] = 5;
        machine.execute_one();
        assert_eq!(machine.registers[1], 255);
        assert_eq!(machine.flag_register(), 0);

        machine.execute_one();
        assert_eq!(machine.registers[1], 4);
        assert_eq!(machine.flag_register(), 1);
    }

    #[test]
    fn test_instr_subtract_xy() {
        let mut machine = Machine::from_instrhex(&[0x8125, 0x8125, 0x8125]);
        machine.registers[1] = 10;
        machine.registers[2] = 5;
        machine.execute_one();
        assert_eq!(machine.registers[1], 5);
        assert_eq!(machine.flag_register(), 1);

        machine.execute_one();
        assert_eq!(machine.registers[1], 0);
        assert_eq!(machine.flag_register(), 1);

        machine.execute_one();
        assert_eq!(machine.registers[1], 251);
        assert_eq!(machine.flag_register(), 0);
    }

    #[test]
    fn test_instr_subtract_yx() {
        let mut machine = Machine::from_instrhex(&[0x8127, 0x8127, 0x8127]);
        machine.registers[1] = 5;
        machine.registers[2] = 10;
        machine.execute_one();
        assert_eq!(machine.registers[1], 5);
        assert_eq!(machine.flag_register(), 1);

        machine.registers[2] = 4;
        machine.execute_one();
        assert_eq!(machine.registers[1], 255);
        assert_eq!(machine.flag_register(), 0);
    }

    #[test]
    fn test_instr_shift_right() {
        let mut machine = Machine::from_instrhex(&[0x8126, 0x8126]);
        machine.registers[1] = 0b0101;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0b010);
        assert_eq!(machine.flag_register(), 1);

        machine.execute_one();
        assert_eq!(machine.registers[1], 0b01);
        assert_eq!(machine.flag_register(), 0);
    }

    #[test]
    fn test_instr_shift_left() {
        let mut machine = Machine::from_instrhex(&[0x812E, 0x812E]);
        machine.registers[1] = 0b10101010;
        machine.execute_one();
        assert_eq!(machine.registers[1], 0b01010100);
        assert_eq!(machine.flag_register(), 1);

        machine.execute_one();
        assert_eq!(machine.registers[1], 0b10101000);
        assert_eq!(machine.flag_register(), 0);
    }

    #[test]
    fn test_instr_jump_with_offset() {
        let mut machine = Machine::from_instrhex(&[0xB012]);
        machine.registers[0] = 5;
        machine.execute_one();
        assert_eq!(machine.program_counter, 0x012 + 5);
    }

    #[test]
    fn test_instr_random() {
        // We can't easily seed the rng, so what we do is generate a few random numbers and check
        // they were anded with 0x0F
        let mut machine = Machine::from_instrhex(&[
            0xC00F,                            // generate random number,
            0x1000 + ROM_START_ADDRESS as u16, // infinite loop to rng above
        ]);
        for _ in 0..10 {
            machine.execute_one();
            assert!(machine.registers[0] < 0x10);
        }
    }
}
