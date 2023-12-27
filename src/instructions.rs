#[derive(Debug)]
pub enum Instruction {
    Zero,
    ClearScreen,
    Jump(u16),
    SetRegister(u8, u8),
    AddToRegister(u8, u8),
    SetIndexRegister(u16),
    Display(u8, u8, u8),
    Unknown(u16),
}

trait NibbleDecoder {
    fn category(&self) -> u8;
    fn vx(&self) -> u8;
    fn vy(&self) -> u8;
    fn n(&self) -> u8;
    fn nn(&self) -> u8;
    fn nnn(&self) -> u16;
}

impl NibbleDecoder for u16 {
    fn category(&self) -> u8 {
        ((self & 0xF000) >> 12) as u8
    }
    fn vx(&self) -> u8 {
        ((self & 0x0F00) >> 8) as u8
    }

    fn vy(&self) -> u8 {
        ((self & 0x00F0) >> 4) as u8
    }

    fn n(&self) -> u8 {
        (self & 0x000F) as u8
    }

    fn nn(&self) -> u8 {
        (self & 0x00FF) as u8
    }

    fn nnn(&self) -> u16 {
        self & 0x0FFF
    }
}

pub fn decode(bytes: u16) -> Instruction {
    if bytes == 0 {
        Instruction::Zero
    } else if bytes == 0x00E0 {
        Instruction::ClearScreen
    } else if bytes.category() == 1 {
        Instruction::Jump(bytes.nnn())
    } else if bytes.category() == 6 {
        Instruction::SetRegister(bytes.vx(), bytes.nn())
    } else if bytes.category() == 7 {
        Instruction::AddToRegister(bytes.vx(), bytes.nn())
    } else if bytes.category() == 0xA {
        Instruction::SetIndexRegister(bytes.nnn())
    } else if bytes.category() == 0xD {
        Instruction::Display(bytes.vx(), bytes.vy(), bytes.n())
    } else {
        Instruction::Unknown(bytes)
    }
}