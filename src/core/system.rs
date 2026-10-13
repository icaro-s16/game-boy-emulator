use crate::core::opcode::Opcode;

pub type Byte = u8;
pub type Word = u16;

const MEMORY_SIZE: usize = 8 * 1024;
const DISPLAY_HEIGHT: usize = 144;
const DISPLAY_WIDGHT: usize = 160;

#[derive(Debug, Clone)]
pub enum Register{
    Union(Box<Register>, Box<Register>),
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    PC,
    SP
}

pub struct System{
    pub wram: [Byte; MEMORY_SIZE],
    pub vram: [Byte; MEMORY_SIZE],
    pub a: Byte,
    pub f: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub sp: Word,
    pub pc: Word,
    pub opcodes_map: [Option<Opcode>; 255],
    pub prefixed_opcodes_map: [Opcode; 255]
}

impl System{
    pub fn fetch_instruction(&mut self) -> Opcode{
        let first_byte = self.vram[self.pc as usize];
        self.pc += 1;
        if first_byte == 0xCB{
            let first_byte = self.vram[self.pc as usize];
            self.pc += 1;
            self.prefixed_opcodes_map[first_byte as usize].clone()
        }
        else{
            self.opcodes_map[first_byte as usize].clone().unwrap()
        }
    }

    pub fn get_zflag(&self) -> bool{
        (self.f & (1 << 7)) != 0
    }

    pub fn set_zflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 7;
        }
        else {
            /* 
             *   f: 10100110
             *
             *   1<<7 = 10000000
             *   !(10000000) = 01111111
             *
             *   f & 01111111
             *
             *   10100110 & 01111111 = 00100110
             */
            self.f &= !(1 << 7);
        }
    }

    pub fn get_nflag(&self) -> bool{
        (self.f & (1 << 6)) != 0
    }

    pub fn set_nflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 6;
        }
        else {
            self.f &= !(1 << 6);
        }
    }

    pub fn get_hflag(&self) -> bool{
        (self.f & (1 << 5)) != 0
    }

    pub fn set_hflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 5;
        }
        else {
            self.f &= !(1 << 5);
        }
    }

    pub fn get_cflag(&self) -> bool{
        (self.f & (1 << 4)) != 0
    }

    pub fn set_cflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 4;
        }
        else {
            self.f &= !(1 << 4);
        }
    }

}





