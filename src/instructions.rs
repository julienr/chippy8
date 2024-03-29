#[derive(Debug)]
pub enum Instruction {
    Zero,
    ClearScreen,
    Jump(u16),
    SetRegToVal(u8, u8),
    AddValToReg(u8, u8),
    SetIndexRegister(u16),
    Display(u8, u8, u8),
    Subroutine(u16),
    Return,
    SkipIfEqualRegVal(u8, u8),
    SkipIfNotEqualRegVal(u8, u8),
    SkipIfEqualRegReg(u8, u8),
    SkipIfNotEqualRegReg(u8, u8),
    Set(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    SubtractXY(u8, u8),
    SubtractYX(u8, u8),
    ShiftLeft(u8, u8),
    ShiftRight(u8, u8),
    JumpWithOffset(u16),
    Random(u8, u8),
    SkipIfKeyPressed(u8),
    SkipIfKeyNotPressed(u8),
    ReadDelayTimer(u8),
    SetDelayTimer(u8),
    SetSoundTimer(u8),
    AddToIndex(u8),
    GetKey(u8),
    FontCharacter(u8),
    ConvertToDecimal(u8),
    RegistersToMemory(u8),
    MemoryToRegisters(u8),
    Unknown(u16, String),
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

pub fn decode(bytes: u16, location_int: &str) -> Instruction {
    if bytes == 0 {
        Instruction::Zero
    } else if bytes == 0x00E0 {
        Instruction::ClearScreen
    } else if bytes == 0x00EE {
        Instruction::Return
    } else if bytes.category() == 1 {
        Instruction::Jump(bytes.nnn())
    } else if bytes.category() == 2 {
        Instruction::Subroutine(bytes.nnn())
    } else if bytes.category() == 3 {
        Instruction::SkipIfEqualRegVal(bytes.vx(), bytes.nn())
    } else if bytes.category() == 4 {
        Instruction::SkipIfNotEqualRegVal(bytes.vx(), bytes.nn())
    } else if bytes.category() == 5 {
        Instruction::SkipIfEqualRegReg(bytes.vx(), bytes.vy())
    } else if bytes.category() == 6 {
        Instruction::SetRegToVal(bytes.vx(), bytes.nn())
    } else if bytes.category() == 7 {
        Instruction::AddValToReg(bytes.vx(), bytes.nn())
    } else if bytes.category() == 8 && bytes.n() == 0 {
        Instruction::Set(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 1 {
        Instruction::Or(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 2 {
        Instruction::And(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 3 {
        Instruction::Xor(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 4 {
        Instruction::Add(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 5 {
        Instruction::SubtractXY(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 6 {
        Instruction::ShiftRight(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 7 {
        Instruction::SubtractYX(bytes.vx(), bytes.vy())
    } else if bytes.category() == 8 && bytes.n() == 0xE {
        Instruction::ShiftLeft(bytes.vx(), bytes.vy())
    } else if bytes.category() == 9 {
        Instruction::SkipIfNotEqualRegReg(bytes.vx(), bytes.vy())
    } else if bytes.category() == 0xA {
        Instruction::SetIndexRegister(bytes.nnn())
    } else if bytes.category() == 0xB {
        Instruction::JumpWithOffset(bytes.nnn())
    } else if bytes.category() == 0xC {
        Instruction::Random(bytes.vx(), bytes.nn())
    } else if bytes.category() == 0xD {
        Instruction::Display(bytes.vx(), bytes.vy(), bytes.n())
    } else if bytes.category() == 0xE && bytes.nn() == 0x9E {
        Instruction::SkipIfKeyPressed(bytes.vx())
    } else if bytes.category() == 0xE && bytes.nn() == 0xA1 {
        Instruction::SkipIfKeyNotPressed(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x07 {
        Instruction::ReadDelayTimer(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x15 {
        Instruction::SetDelayTimer(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x18 {
        Instruction::SetSoundTimer(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x1E {
        Instruction::AddToIndex(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x0A {
        Instruction::GetKey(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x29 {
        Instruction::FontCharacter(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x33 {
        Instruction::ConvertToDecimal(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x55 {
        Instruction::RegistersToMemory(bytes.vx())
    } else if bytes.category() == 0xF && bytes.nn() == 0x65 {
        Instruction::MemoryToRegisters(bytes.vx())
    } else {
        Instruction::Unknown(bytes, location_int.to_string())
    }
}
