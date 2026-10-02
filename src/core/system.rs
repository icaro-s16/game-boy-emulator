use crate::core::system_types::{
    Byte, 
    SType, 
    Word
};

const MEMORY_SIZE: usize = 8 * 1024;
const DISPLAY_HEIGHT: usize = 144;
const DISPLAY_WIDGHT: usize = 160;

#[derive(
    Debug, 
    Clone, 
    Copy
)]
pub(crate) enum Register{
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    SP,
    PC
}

#[derive(
    Debug, 
    Clone, 
    Copy
)]
pub(crate) enum RegisterSet{
    Unary(Register),
    Binary(Register, Register)
}

pub(crate) struct System{
    pub(crate) wram: [u8; MEMORY_SIZE],
    pub(crate) vram: [u8; MEMORY_SIZE],
    pub(crate) a: Byte,
    pub(crate) f: Byte,
    pub(crate) b: Byte,
    pub(crate) c: Byte,
    pub(crate) d: Byte,
    pub(crate) e: Byte,
    pub(crate) h: Byte,
    pub(crate) l: Byte,
    pub(crate) sp: Word,
    pub(crate) pc: Word
}

impl PartialEq for Register{
    fn eq(&self, other: &Self) -> bool {
        match (self, other){
            (Register::A,Register::A) => true,
            (Register::B,Register::B) => true,
            (Register::C,Register::C) => true,
            (Register::D,Register::D) => true,
            (Register::E,Register::E) => true,
            (Register::F,Register::F) => true,
            (Register::H,Register::H) => true,
            (Register::L,Register::L) => true,
            (Register::SP,Register::SP) => true,
            (Register::PC,Register::PC) => true,
            _ => false
        }
    }
}

impl Register{
    fn compound_register(r1: Register, r2: Register) -> RegisterSet{
        if 
            r1 == Register::PC || 
            r1 == Register::SP ||
            r2 == Register::PC ||
            r2 == Register::SP
        {
            panic!("ERROR: Invalid compound register");
        } 
        RegisterSet::Binary(r1, r2)
    }
}

impl System{
    fn get_sregister(&self, register: Register) -> SType{
        match register{
            Register::A  => SType::Byte(self.a),
            Register::B  => SType::Byte(self.b),
            Register::C  => SType::Byte(self.c),
            Register::D  => SType::Byte(self.d),
            Register::E  => SType::Byte(self.e),
            Register::F  => SType::Byte(self.f),
            Register::H  => SType::Byte(self.h),
            Register::L  => SType::Byte(self.l),
            Register::SP => SType::Word(self.sp),
            Register::PC => SType::Word(self.pc),
            _ => panic!()
        }
    }

    pub(crate) fn get_register(&self, register: RegisterSet) -> SType{
        match register {
            RegisterSet::Binary(r1, r2) => SType::Word(
                Word::from_bytes(
                self.get_sregister(r1).get_byte(), 
                self.get_sregister(r2).get_byte()
                )
            ),
            RegisterSet::Unary(r) => self.get_sregister(r)
        }
    }

    fn set_sregister(&mut self, register: Register, val: SType){
        match register{
            Register::A  => {self.a = val.get_byte();},
            Register::B  => {self.b = val.get_byte();},
            Register::C  => {self.c = val.get_byte();},
            Register::D  => {self.d = val.get_byte();},
            Register::E  => {self.e = val.get_byte();},
            Register::F  => {self.f = val.get_byte();},
            Register::H  => {self.h = val.get_byte();},
            Register::L  => {self.l = val.get_byte();},
            Register::SP => {self.sp = val.get_word();},
            Register::PC => {self.pc = val.get_word();},
            _ => panic!()
        }
    } 

    pub(crate) fn set_register(&mut self, register: RegisterSet, val: SType){
        match register{
            RegisterSet::Binary(r1,r2) => {
                let (hbyte, lbyte) = val.get_word().to_bytes();
                self.set_sregister(r1, SType::Byte(hbyte));
                self.set_sregister(r2, SType::Byte(lbyte));

            },
            RegisterSet::Unary(r) => {
                self.set_sregister(r, SType::Byte(val.get_byte()));
            }
        }
    }

}


