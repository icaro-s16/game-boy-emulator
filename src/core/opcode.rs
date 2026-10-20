use core::panic;
use std::{any::Any, unreachable};

use crate::core::system::{
    Flag, Register, System
};

#[derive(Clone)]
pub struct Opcode{
    pub raw: u8,
    pub mnemonic: &'static str,
    pub operand: OperandSet,
    pub cycles: Vec<u8>
}
pub trait Operand{
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn Operand>; 

    fn get_type(&self) -> OperandType;

    /* NOTE: Every operand value can be interpreted 
     * as an usize, and after that can be casted to a
     * differente type, this allow us to use any
     * operand as a index even when we use dyn Trait.
     */
    fn get_value(&self, system: &mut System) -> usize;
    
    fn update(&self, _value: usize, _system: &mut System){
        panic!("[ERROR] The operand can not be updated")
    }
}

#[derive(Clone)]
pub enum OperandSet{
    Binary(Box<dyn Operand>, Box<dyn Operand>),
    Unary(Box<dyn Operand>),
    Nullary
}

#[derive(Clone, Copy)]
pub struct U16;
#[derive(Clone, Copy)]
pub struct U8;
#[derive(Clone, Copy)]
pub struct I8;
#[derive(Clone, Copy)]
pub struct Literal(usize);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OperandType{
    Address,
    Register,
    U16,
    U8,
    I8,
    Flag,

    /* NOTE: The literal operand type can be 
     * an fixed address (e.g. RST 0x18), or can
     * be fixed index (e.g. SET 0, [HL])
     */
    Literal
}

#[derive(Clone)]
pub struct Address(Box<dyn Operand>);

impl Opcode{
    pub fn new(raw: u8, mnemonic: &'static str, operand: OperandSet, cycles: Vec<u8>) -> Self{
        Self {raw, mnemonic, operand, cycles }
    }
}

impl Clone for Box<dyn Operand>{
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Operand for Address{
    fn as_any(&self) -> &dyn Any{
        self
    }

    fn clone_box(&self) -> Box<dyn Operand>{
        Box::new(self.clone())
    }

    fn get_type(&self) -> OperandType{
        OperandType::Address
    }

    fn get_value(&self, system: &mut System) -> usize{
        if 
            self.0.get_type() == OperandType::Address   ||
            self.0.get_type() == OperandType::Flag      ||
            self.0.get_type() == OperandType::Literal
        {
            panic!("[ERROR] The value inside of an address can not be an address, a flag or a literal");
        }

        system.wram[
            self.0.get_value(system)
        ] as usize
    }

    fn update(&self, value: usize, system: &mut System){
        system.wram[self.0.get_value(system)] = value as u8;
    }
}

impl Address{
    pub fn get_value_with_offset(&self, offset: usize, system: &mut System) -> usize{
        if 
            self.0.get_type() == OperandType::Address   ||
            self.0.get_type() == OperandType::Flag      ||
            self.0.get_type() == OperandType::Literal
        {
            panic!("[ERROR] The value inside of an address can not be an address, a flag or a literal");
        }

        system.wram[
            self.0.get_value(system) + offset
        ] as usize
    }

    pub fn update_with_offset(&self, value: usize, offset: usize, system: &mut System){
        system.wram[self.0.get_value(system) + offset] = value as u8;
    }
}

impl Operand for I8{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> OperandType{
        OperandType::I8
    }

    fn get_value(&self, system: &mut System) -> usize{
        let value = system.vram[system.pc as usize].clone() as usize;
        system.pc += 1;
        value
    }
}

impl Operand for U8{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> OperandType{
        OperandType::U8
    }

    fn get_value(&self, system: &mut System) -> usize{
        let value = system.vram[system.pc as usize].clone() as usize;
        system.pc += 1;
        value
    }
}

impl Operand for U16{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
    
    fn get_type(&self) -> OperandType{
        OperandType::U16
    }

    fn get_value(&self, system: &mut System) -> usize{
        let v1 = system.vram[system.pc as usize].clone();
        system.pc += 1;
        let v2 = system.vram[system.pc as usize].clone();
        system.pc += 1;

        /* NOTE: Game Boy use little endian. 
         */
        i16::from_be_bytes([v2, v1]) as usize
    }
}

impl Operand for Literal{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
    
    fn get_type(&self) -> OperandType{
        OperandType::Literal
    }

    fn get_value(&self, _system: &mut System) -> usize{
        self.0 as usize
    }
}

impl Operand for Flag{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand>{
        Box::new(self.clone())
    } 

    fn get_type(&self) -> OperandType{
        OperandType::Flag
    }

    /* NOTE: There is two types of operands N(flag) and (flag)
     * the N(flag) is the not conditional, and is represent 
     * by the false value inside of a flag (e.g. Flag::Z(false) == NZ)  
     */ 
    fn get_value(&self, system: &mut System) -> usize{
        match self{
            Flag::Z => {
                system.get_zflag() as usize
            },
            Flag::NZ => {
                !system.get_zflag() as usize
            }
            Flag::C => {
                system.get_cflag() as usize
            },
            Flag::NC => {
                !system.get_cflag() as usize
            }
            Flag::N => {
                system.get_zflag() as usize
            },
            Flag::H => {
                system.get_zflag() as usize
            }
        }
    }
}

impl Operand for Register{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
    
    fn get_type(&self) -> OperandType{
        OperandType::Register
    }

    fn get_value(&self, system: &mut System) -> usize{
        match self{
            Register::Union(r1, r2) => {
                let hbyte = match r1.as_ref(){
                    Register::A => system.a.clone(),
                    Register::B => system.b.clone(),
                    Register::C => system.c.clone(),
                    Register::D => system.d.clone(),
                    Register::E => system.e.clone(),
                    Register::F => system.f.clone(),
                    Register::H => system.h.clone(),
                    Register::L => system.l.clone(),
                    _ => panic!("[ERROR] An union register can not be an recursive type or the PC/SP")
                };
                let lbyte = match r2.as_ref(){
                    Register::A => system.a.clone(),
                    Register::B => system.b.clone(),
                    Register::C => system.c.clone(),
                    Register::D => system.d.clone(),
                    Register::E => system.e.clone(),
                    Register::F => system.f.clone(),
                    Register::H => system.h.clone(),
                    Register::L => system.l.clone(),
                    _ => panic!("[ERROR] An union register can not be an recursive type or the PC/SP")
                };
                i16::from_be_bytes([hbyte, lbyte]) as usize
            },
            Register::A => {system.a as usize},
            Register::B => {system.b as usize},
            Register::C => {system.c as usize},
            Register::D => {system.d as usize},
            Register::E => {system.e as usize},
            Register::F => {system.f as usize},
            Register::H => {system.h as usize},
            Register::L => {system.l as usize},
            _ => panic!("[ERROR] SP and PC can not be an operand register")
        }
    }

    fn update(&self, value: usize, system: &mut System){
        let word = value as u16;
        let bytes = word.to_be_bytes();
        match self {
            Register::Union(r1, r2) => {
                match r1.as_ref(){
                    Register::A => system.a = bytes[0],
                    Register::B => system.b = bytes[0],
                    Register::C => system.c = bytes[0],
                    Register::D => system.d = bytes[0],
                    Register::E => system.e = bytes[0],
                    Register::F => system.f = bytes[0],
                    Register::H => system.h = bytes[0],
                    Register::L => system.l = bytes[0],
                    _ => {panic!("[ERROR] An union register can not have another union or the PC/SP register")}
                }
                match r2.as_ref(){
                    Register::A => system.a = bytes[1],
                    Register::B => system.b = bytes[1],
                    Register::C => system.c = bytes[1],
                    Register::D => system.d = bytes[1],
                    Register::E => system.e = bytes[1],
                    Register::F => system.f = bytes[1],
                    Register::H => system.h = bytes[1],
                    Register::L => system.l = bytes[1],
                    _ => {panic!("[ERROR] An union register can not have another union or the PC/SP register")}
                }
            },
            Register::A => system.a = bytes[1],
            Register::B => system.b = bytes[1],
            Register::C => system.c = bytes[1],
            Register::D => system.d = bytes[1],
            Register::E => system.e = bytes[1],
            Register::F => system.f = bytes[1],
            Register::H => system.h = bytes[1],
            Register::L => system.l = bytes[1],
            Register::SP => system.sp = word,
            Register::PC => panic!("[ERROR] PC can not be used as an operand")
        }
    }
}

pub fn is_16bit_operand(operand: &dyn Operand) -> bool{
    match operand.get_type(){
        OperandType::Register => {
            let reg = 
                if let Some(reg) = operand.as_any().downcast_ref::<Register>() {
                    reg
                }
                else {
                    unreachable!()
                };

            if let Register::Union(_, _) = reg {true} else {false}
        },
        OperandType::U16 => true,
        _ => false
    }
}

#[macro_export]
macro_rules! operand_arg {
    ([($op1:expr, $op2:expr)]) => {
        Box::new(Address(Box::new(Register::Union(Box::new($op1), Box::new($op2)))))
    };
    (($op1:expr, $op2:expr)) => {
        Box::new(Register::Union(Box::new($op1), Box::new($op2)))
    };
    ([$op:expr]) => {
        Box::new(Address(Box::new($op)))
    };
    ($op:expr) => {
        Box::new($op)
    }
    
}

#[macro_export]
macro_rules! operand {
    ($op1:tt, $op2:tt) => {
        OperandSet::Binary(operand_arg!($op1), operand_arg!($op2))
    };
    ($op:tt) => {
        OperandSet::Unary(operand_arg!($op))
    };
    () => {
        OperandSet::Nullary
    }
}

pub fn build_unprefixed_map() -> [Option<Opcode>; 256] {
    [
        Some(Opcode::new(0x00, "NOP ", operand!(), vec![4])),
        Some(Opcode::new(0x01, "LD BC, u16", operand!(((Register::B),(Register::C)),U16), vec![12])),
        Some(Opcode::new(0x02, "LD [BC], A", operand!([((Register::B),(Register::C))],(Register::A)), vec![8])),
        Some(Opcode::new(0x03, "INC BC", operand!(((Register::B),(Register::C))), vec![8])),
        Some(Opcode::new(0x04, "INC B", operand!((Register::B)), vec![4])),
        Some(Opcode::new(0x05, "DEC B", operand!((Register::B)), vec![4])),
        Some(Opcode::new(0x06, "LD B, u8", operand!((Register::B),U8), vec![8])),
        Some(Opcode::new(0x07, "RLCA ", operand!(), vec![4])),
        Some(Opcode::new(0x08, "LD [u16], SP", operand!([U16],(Register::SP)), vec![20])),
        Some(Opcode::new(0x09, "ADD HL, BC", operand!(((Register::H),(Register::L)),((Register::B),(Register::C))), vec![8])),
        Some(Opcode::new(0x0A, "LD A, [BC]", operand!((Register::A),[((Register::B),(Register::C))]), vec![8])),
        Some(Opcode::new(0x0B, "DEC BC", operand!(((Register::B),(Register::C))), vec![8])),
        Some(Opcode::new(0x0C, "INC C", operand!((Register::C)), vec![4])),
        Some(Opcode::new(0x0D, "DEC C", operand!((Register::C)), vec![4])),
        Some(Opcode::new(0x0E, "LD C, u8", operand!((Register::C),U8), vec![8])),
        Some(Opcode::new(0x0F, "RRCA ", operand!(), vec![4])),
        Some(Opcode::new(0x10, "STOP u8", operand!(U8), vec![4])),
        Some(Opcode::new(0x11, "LD DE, u16", operand!(((Register::D),(Register::E)),U16), vec![12])),
        Some(Opcode::new(0x12, "LD [DE], A", operand!([((Register::D),(Register::E))],(Register::A)), vec![8])),
        Some(Opcode::new(0x13, "INC DE", operand!(((Register::D),(Register::E))), vec![8])),
        Some(Opcode::new(0x14, "INC D", operand!((Register::D)), vec![4])),
        Some(Opcode::new(0x15, "DEC D", operand!((Register::D)), vec![4])),
        Some(Opcode::new(0x16, "LD D, u8", operand!((Register::D),U8), vec![8])),
        Some(Opcode::new(0x17, "RLA ", operand!(), vec![4])),
        Some(Opcode::new(0x18, "JR i8", operand!(I8), vec![12])),
        Some(Opcode::new(0x19, "ADD HL, DE", operand!(((Register::H),(Register::L)),((Register::D),(Register::E))), vec![8])),
        Some(Opcode::new(0x1A, "LD A, [DE]", operand!((Register::A),[((Register::D),(Register::E))]), vec![8])),
        Some(Opcode::new(0x1B, "DEC DE", operand!(((Register::D),(Register::E))), vec![8])),
        Some(Opcode::new(0x1C, "INC E", operand!((Register::E)), vec![4])),
        Some(Opcode::new(0x1D, "DEC E", operand!((Register::E)), vec![4])),
        Some(Opcode::new(0x1E, "LD E, u8", operand!((Register::E),U8), vec![8])),
        Some(Opcode::new(0x1F, "RRA ", operand!(), vec![4])),
        Some(Opcode::new(0x20, "JR NZ, i8", operand!((Flag::NZ),I8), vec![12, 8])),
        Some(Opcode::new(0x21, "LD HL, u16", operand!(((Register::H),(Register::L)),U16), vec![12])),
        Some(Opcode::new(0x22, "LD [HL+], A", operand!([((Register::H),(Register::L))],(Register::A)), vec![8])),
        Some(Opcode::new(0x23, "INC HL", operand!(((Register::H),(Register::L))), vec![8])),
        Some(Opcode::new(0x24, "INC H", operand!((Register::H)), vec![4])),
        Some(Opcode::new(0x25, "DEC H", operand!((Register::H)), vec![4])),
        Some(Opcode::new(0x26, "LD H, u8", operand!((Register::H),U8), vec![8])),
        Some(Opcode::new(0x27, "DAA ", operand!(), vec![4])),
        Some(Opcode::new(0x28, "JR Z, i8", operand!((Flag::Z),I8), vec![12, 8])),
        Some(Opcode::new(0x29, "ADD HL, HL", operand!(((Register::H),(Register::L)),((Register::H),(Register::L))), vec![8])),
        Some(Opcode::new(0x2A, "LD A, [HL+]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x2B, "DEC HL", operand!(((Register::H),(Register::L))), vec![8])),
        Some(Opcode::new(0x2C, "INC L", operand!((Register::L)), vec![4])),
        Some(Opcode::new(0x2D, "DEC L", operand!((Register::L)), vec![4])),
        Some(Opcode::new(0x2E, "LD L, u8", operand!((Register::L),U8), vec![8])),
        Some(Opcode::new(0x2F, "CPL ", operand!(), vec![4])),
        Some(Opcode::new(0x30, "JR NC, i8", operand!((Flag::NC),I8), vec![12, 8])),
        Some(Opcode::new(0x31, "LD SP, u16", operand!((Register::SP),U16), vec![12])),
        Some(Opcode::new(0x32, "LD [HL-], A", operand!([((Register::H),(Register::L))],(Register::A)), vec![8])),
        Some(Opcode::new(0x33, "INC SP", operand!((Register::SP)), vec![8])),
        Some(Opcode::new(0x34, "INC [HL]", operand!([((Register::H),(Register::L))]), vec![12])),
        Some(Opcode::new(0x35, "DEC [HL]", operand!([((Register::H),(Register::L))]), vec![12])),
        Some(Opcode::new(0x36, "LD [HL], u8", operand!([((Register::H),(Register::L))],U8), vec![12])),
        Some(Opcode::new(0x37, "SCF ", operand!(), vec![4])),
        Some(Opcode::new(0x38, "JR C, i8", operand!((Flag::C),I8), vec![12, 8])),
        Some(Opcode::new(0x39, "ADD HL, SP", operand!(((Register::H),(Register::L)),(Register::SP)), vec![8])),
        Some(Opcode::new(0x3A, "LD A, [HL-]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x3B, "DEC SP", operand!((Register::SP)), vec![8])),
        Some(Opcode::new(0x3C, "INC A", operand!((Register::A)), vec![4])),
        Some(Opcode::new(0x3D, "DEC A", operand!((Register::A)), vec![4])),
        Some(Opcode::new(0x3E, "LD A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0x3F, "CCF ", operand!(), vec![4])),
        Some(Opcode::new(0x40, "LD B, B", operand!((Register::B),(Register::B)), vec![4])),
        Some(Opcode::new(0x41, "LD B, C", operand!((Register::B),(Register::C)), vec![4])),
        Some(Opcode::new(0x42, "LD B, D", operand!((Register::B),(Register::D)), vec![4])),
        Some(Opcode::new(0x43, "LD B, E", operand!((Register::B),(Register::E)), vec![4])),
        Some(Opcode::new(0x44, "LD B, H", operand!((Register::B),(Register::H)), vec![4])),
        Some(Opcode::new(0x45, "LD B, L", operand!((Register::B),(Register::L)), vec![4])),
        Some(Opcode::new(0x46, "LD B, [HL]", operand!((Register::B),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x47, "LD B, A", operand!((Register::B),(Register::A)), vec![4])),
        Some(Opcode::new(0x48, "LD C, B", operand!((Register::C),(Register::B)), vec![4])),
        Some(Opcode::new(0x49, "LD C, C", operand!((Register::C),(Register::C)), vec![4])),
        Some(Opcode::new(0x4A, "LD C, D", operand!((Register::C),(Register::D)), vec![4])),
        Some(Opcode::new(0x4B, "LD C, E", operand!((Register::C),(Register::E)), vec![4])),
        Some(Opcode::new(0x4C, "LD C, H", operand!((Register::C),(Register::H)), vec![4])),
        Some(Opcode::new(0x4D, "LD C, L", operand!((Register::C),(Register::L)), vec![4])),
        Some(Opcode::new(0x4E, "LD C, [HL]", operand!((Register::C),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x4F, "LD C, A", operand!((Register::C),(Register::A)), vec![4])),
        Some(Opcode::new(0x50, "LD D, B", operand!((Register::D),(Register::B)), vec![4])),
        Some(Opcode::new(0x51, "LD D, C", operand!((Register::D),(Register::C)), vec![4])),
        Some(Opcode::new(0x52, "LD D, D", operand!((Register::D),(Register::D)), vec![4])),
        Some(Opcode::new(0x53, "LD D, E", operand!((Register::D),(Register::E)), vec![4])),
        Some(Opcode::new(0x54, "LD D, H", operand!((Register::D),(Register::H)), vec![4])),
        Some(Opcode::new(0x55, "LD D, L", operand!((Register::D),(Register::L)), vec![4])),
        Some(Opcode::new(0x56, "LD D, [HL]", operand!((Register::D),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x57, "LD D, A", operand!((Register::D),(Register::A)), vec![4])),
        Some(Opcode::new(0x58, "LD E, B", operand!((Register::E),(Register::B)), vec![4])),
        Some(Opcode::new(0x59, "LD E, C", operand!((Register::E),(Register::C)), vec![4])),
        Some(Opcode::new(0x5A, "LD E, D", operand!((Register::E),(Register::D)), vec![4])),
        Some(Opcode::new(0x5B, "LD E, E", operand!((Register::E),(Register::E)), vec![4])),
        Some(Opcode::new(0x5C, "LD E, H", operand!((Register::E),(Register::H)), vec![4])),
        Some(Opcode::new(0x5D, "LD E, L", operand!((Register::E),(Register::L)), vec![4])),
        Some(Opcode::new(0x5E, "LD E, [HL]", operand!((Register::E),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x5F, "LD E, A", operand!((Register::E),(Register::A)), vec![4])),
        Some(Opcode::new(0x60, "LD H, B", operand!((Register::H),(Register::B)), vec![4])),
        Some(Opcode::new(0x61, "LD H, C", operand!((Register::H),(Register::C)), vec![4])),
        Some(Opcode::new(0x62, "LD H, D", operand!((Register::H),(Register::D)), vec![4])),
        Some(Opcode::new(0x63, "LD H, E", operand!((Register::H),(Register::E)), vec![4])),
        Some(Opcode::new(0x64, "LD H, H", operand!((Register::H),(Register::H)), vec![4])),
        Some(Opcode::new(0x65, "LD H, L", operand!((Register::H),(Register::L)), vec![4])),
        Some(Opcode::new(0x66, "LD H, [HL]", operand!((Register::H),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x67, "LD H, A", operand!((Register::H),(Register::A)), vec![4])),
        Some(Opcode::new(0x68, "LD L, B", operand!((Register::L),(Register::B)), vec![4])),
        Some(Opcode::new(0x69, "LD L, C", operand!((Register::L),(Register::C)), vec![4])),
        Some(Opcode::new(0x6A, "LD L, D", operand!((Register::L),(Register::D)), vec![4])),
        Some(Opcode::new(0x6B, "LD L, E", operand!((Register::L),(Register::E)), vec![4])),
        Some(Opcode::new(0x6C, "LD L, H", operand!((Register::L),(Register::H)), vec![4])),
        Some(Opcode::new(0x6D, "LD L, L", operand!((Register::L),(Register::L)), vec![4])),
        Some(Opcode::new(0x6E, "LD L, [HL]", operand!((Register::L),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x6F, "LD L, A", operand!((Register::L),(Register::A)), vec![4])),
        Some(Opcode::new(0x70, "LD [HL], B", operand!([((Register::H),(Register::L))],(Register::B)), vec![8])),
        Some(Opcode::new(0x71, "LD [HL], C", operand!([((Register::H),(Register::L))],(Register::C)), vec![8])),
        Some(Opcode::new(0x72, "LD [HL], D", operand!([((Register::H),(Register::L))],(Register::D)), vec![8])),
        Some(Opcode::new(0x73, "LD [HL], E", operand!([((Register::H),(Register::L))],(Register::E)), vec![8])),
        Some(Opcode::new(0x74, "LD [HL], H", operand!([((Register::H),(Register::L))],(Register::H)), vec![8])),
        Some(Opcode::new(0x75, "LD [HL], L", operand!([((Register::H),(Register::L))],(Register::L)), vec![8])),
        Some(Opcode::new(0x76, "HALT ", operand!(), vec![4])),
        Some(Opcode::new(0x77, "LD [HL], A", operand!([((Register::H),(Register::L))],(Register::A)), vec![8])),
        Some(Opcode::new(0x78, "LD A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0x79, "LD A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0x7A, "LD A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0x7B, "LD A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0x7C, "LD A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0x7D, "LD A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0x7E, "LD A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x7F, "LD A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0x80, "ADD A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0x81, "ADD A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0x82, "ADD A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0x83, "ADD A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0x84, "ADD A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0x85, "ADD A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0x86, "ADD A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x87, "ADD A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0x88, "ADC A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0x89, "ADC A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0x8A, "ADC A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0x8B, "ADC A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0x8C, "ADC A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0x8D, "ADC A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0x8E, "ADC A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x8F, "ADC A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0x90, "SUB A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0x91, "SUB A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0x92, "SUB A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0x93, "SUB A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0x94, "SUB A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0x95, "SUB A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0x96, "SUB A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x97, "SUB A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0x98, "SBC A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0x99, "SBC A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0x9A, "SBC A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0x9B, "SBC A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0x9C, "SBC A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0x9D, "SBC A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0x9E, "SBC A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0x9F, "SBC A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0xA0, "AND A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0xA1, "AND A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0xA2, "AND A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0xA3, "AND A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0xA4, "AND A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0xA5, "AND A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0xA6, "AND A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0xA7, "AND A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0xA8, "XOR A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0xA9, "XOR A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0xAA, "XOR A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0xAB, "XOR A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0xAC, "XOR A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0xAD, "XOR A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0xAE, "XOR A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0xAF, "XOR A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0xB0, "OR A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0xB1, "OR A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0xB2, "OR A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0xB3, "OR A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0xB4, "OR A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0xB5, "OR A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0xB6, "OR A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0xB7, "OR A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0xB8, "CP A, B", operand!((Register::A),(Register::B)), vec![4])),
        Some(Opcode::new(0xB9, "CP A, C", operand!((Register::A),(Register::C)), vec![4])),
        Some(Opcode::new(0xBA, "CP A, D", operand!((Register::A),(Register::D)), vec![4])),
        Some(Opcode::new(0xBB, "CP A, E", operand!((Register::A),(Register::E)), vec![4])),
        Some(Opcode::new(0xBC, "CP A, H", operand!((Register::A),(Register::H)), vec![4])),
        Some(Opcode::new(0xBD, "CP A, L", operand!((Register::A),(Register::L)), vec![4])),
        Some(Opcode::new(0xBE, "CP A, [HL]", operand!((Register::A),[((Register::H),(Register::L))]), vec![8])),
        Some(Opcode::new(0xBF, "CP A, A", operand!((Register::A),(Register::A)), vec![4])),
        Some(Opcode::new(0xC0, "RET NZ", operand!((Flag::NZ)), vec![20, 8])),
        Some(Opcode::new(0xC1, "POP BC", operand!(((Register::B),(Register::C))), vec![12])),
        Some(Opcode::new(0xC2, "JP NZ, u16", operand!((Flag::NZ),U16), vec![16, 12])),
        Some(Opcode::new(0xC3, "JP u16", operand!(U16), vec![16])),
        Some(Opcode::new(0xC4, "CALL NZ, u16", operand!((Flag::NZ),U16), vec![24, 12])),
        Some(Opcode::new(0xC5, "PUSH BC", operand!(((Register::B),(Register::C))), vec![16])),
        Some(Opcode::new(0xC6, "ADD A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xC7, "RST $00", operand!((Literal(0x00))), vec![16])),
        Some(Opcode::new(0xC8, "RET Z", operand!((Flag::Z)), vec![20, 8])),
        Some(Opcode::new(0xC9, "RET ", operand!(), vec![16])),
        Some(Opcode::new(0xCA, "JP Z, u16", operand!((Flag::Z),U16), vec![16, 12])),
        Some(Opcode::new(0xCB, "PREFIX ", operand!(), vec![4])),
        Some(Opcode::new(0xCC, "CALL Z, u16", operand!((Flag::Z),U16), vec![24, 12])),
        Some(Opcode::new(0xCD, "CALL u16", operand!(U16), vec![24])),
        Some(Opcode::new(0xCE, "ADC A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xCF, "RST $08", operand!((Literal(0x08))), vec![16])),
        Some(Opcode::new(0xD0, "RET NC", operand!((Flag::NC)), vec![20, 8])),
        Some(Opcode::new(0xD1, "POP DE", operand!(((Register::D),(Register::E))), vec![12])),
        Some(Opcode::new(0xD2, "JP NC, u16", operand!((Flag::NC),U16), vec![16, 12])),
        None,
        Some(Opcode::new(0xD4, "CALL NC, u16", operand!((Flag::NC),U16), vec![24, 12])),
        Some(Opcode::new(0xD5, "PUSH DE", operand!(((Register::D),(Register::E))), vec![16])),
        Some(Opcode::new(0xD6, "SUB A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xD7, "RST $10", operand!((Literal(0x10))), vec![16])),
        Some(Opcode::new(0xD8, "RET C", operand!((Flag::C)), vec![20, 8])),
        Some(Opcode::new(0xD9, "RETI ", operand!(), vec![16])),
        Some(Opcode::new(0xDA, "JP C, u16", operand!((Flag::C),U16), vec![16, 12])),
        None,
        Some(Opcode::new(0xDC, "CALL C, u16", operand!((Flag::C),U16), vec![24, 12])),
        None,
        Some(Opcode::new(0xDE, "SBC A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xDF, "RST $18", operand!((Literal(0x18))), vec![16])),
        Some(Opcode::new(0xE0, "LDH [u8], A", operand!([U8],(Register::A)), vec![12])),
        Some(Opcode::new(0xE1, "POP HL", operand!(((Register::H),(Register::L))), vec![12])),
        Some(Opcode::new(0xE2, "LDH [C], A", operand!([(Register::C)],(Register::A)), vec![8])),
        None,
        None,
        Some(Opcode::new(0xE5, "PUSH HL", operand!(((Register::H),(Register::L))), vec![16])),
        Some(Opcode::new(0xE6, "AND A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xE7, "RST $20", operand!((Literal(0x20))), vec![16])),
        Some(Opcode::new(0xE8, "ADD SP, i8", operand!((Register::SP),I8), vec![16])),
        Some(Opcode::new(0xE9, "JP HL", operand!(((Register::H),(Register::L))), vec![4])),
        Some(Opcode::new(0xEA, "LD [u16], A", operand!([U16],(Register::A)), vec![16])),
        None,
        None,
        None,
        Some(Opcode::new(0xEE, "XOR A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xEF, "RST $28", operand!((Literal(0x28))), vec![16])),
        Some(Opcode::new(0xF0, "LDH A, [u8]", operand!((Register::A),[U8]), vec![12])),
        Some(Opcode::new(0xF1, "POP AF", operand!(((Register::A),(Register::F))), vec![12])),
        Some(Opcode::new(0xF2, "LDH A, [C]", operand!((Register::A),[(Register::C)]), vec![8])),
        Some(Opcode::new(0xF3, "DI ", operand!(), vec![4])),
        None,
        Some(Opcode::new(0xF5, "PUSH AF", operand!(((Register::A),(Register::F))), vec![16])),
        Some(Opcode::new(0xF6, "OR A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xF7, "RST $30", operand!((Literal(0x30))), vec![16])),
        Some(Opcode::new(0xF8, "LD HL, SP + i8", operand!(((Register::H),(Register::L)), I8), vec![12])),
        Some(Opcode::new(0xF9, "LD SP, HL", operand!((Register::SP),((Register::H),(Register::L))), vec![8])),
        Some(Opcode::new(0xFA, "LD A, [u16]", operand!((Register::A),[U16]), vec![16])),
        Some(Opcode::new(0xFB, "EI ", operand!(), vec![4])),
        None,
        None,
        Some(Opcode::new(0xFE, "CP A, u8", operand!((Register::A),U8), vec![8])),
        Some(Opcode::new(0xFF, "RST $38", operand!((Literal(0x38))), vec![16]))
    ]
}

pub fn build_prefixed_map() -> [Opcode; 256] {
    [  
        Opcode::new(0x00, "RLC B", operand!((Register::B)), vec![8]),
        Opcode::new(0x01, "RLC C", operand!((Register::C)), vec![8]),
        Opcode::new(0x02, "RLC D", operand!((Register::D)), vec![8]),
        Opcode::new(0x03, "RLC E", operand!((Register::E)), vec![8]),
        Opcode::new(0x04, "RLC H", operand!((Register::H)), vec![8]),
        Opcode::new(0x05, "RLC L", operand!((Register::L)), vec![8]),
        Opcode::new(0x06, "RLC [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x07, "RLC A", operand!((Register::A)), vec![8]),
        Opcode::new(0x08, "RRC B", operand!((Register::B)), vec![8]),
        Opcode::new(0x09, "RRC C", operand!((Register::C)), vec![8]),
        Opcode::new(0x0A, "RRC D", operand!((Register::D)), vec![8]),
        Opcode::new(0x0B, "RRC E", operand!((Register::E)), vec![8]),
        Opcode::new(0x0C, "RRC H", operand!((Register::H)), vec![8]),
        Opcode::new(0x0D, "RRC L", operand!((Register::L)), vec![8]),
        Opcode::new(0x0E, "RRC [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x0F, "RRC A", operand!((Register::A)), vec![8]),
        Opcode::new(0x10, "RL B", operand!((Register::B)), vec![8]),
        Opcode::new(0x11, "RL C", operand!((Register::C)), vec![8]),
        Opcode::new(0x12, "RL D", operand!((Register::D)), vec![8]),
        Opcode::new(0x13, "RL E", operand!((Register::E)), vec![8]),
        Opcode::new(0x14, "RL H", operand!((Register::H)), vec![8]),
        Opcode::new(0x15, "RL L", operand!((Register::L)), vec![8]),
        Opcode::new(0x16, "RL [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x17, "RL A", operand!((Register::A)), vec![8]),
        Opcode::new(0x18, "RR B", operand!((Register::B)), vec![8]),
        Opcode::new(0x19, "RR C", operand!((Register::C)), vec![8]),
        Opcode::new(0x1A, "RR D", operand!((Register::D)), vec![8]),
        Opcode::new(0x1B, "RR E", operand!((Register::E)), vec![8]),
        Opcode::new(0x1C, "RR H", operand!((Register::H)), vec![8]),
        Opcode::new(0x1D, "RR L", operand!((Register::L)), vec![8]),
        Opcode::new(0x1E, "RR [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x1F, "RR A", operand!((Register::A)), vec![8]),
        Opcode::new(0x20, "SLA B", operand!((Register::B)), vec![8]),
        Opcode::new(0x21, "SLA C", operand!((Register::C)), vec![8]),
        Opcode::new(0x22, "SLA D", operand!((Register::D)), vec![8]),
        Opcode::new(0x23, "SLA E", operand!((Register::E)), vec![8]),
        Opcode::new(0x24, "SLA H", operand!((Register::H)), vec![8]),
        Opcode::new(0x25, "SLA L", operand!((Register::L)), vec![8]),
        Opcode::new(0x26, "SLA [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x27, "SLA A", operand!((Register::A)), vec![8]),
        Opcode::new(0x28, "SRA B", operand!((Register::B)), vec![8]),
        Opcode::new(0x29, "SRA C", operand!((Register::C)), vec![8]),
        Opcode::new(0x2A, "SRA D", operand!((Register::D)), vec![8]),
        Opcode::new(0x2B, "SRA E", operand!((Register::E)), vec![8]),
        Opcode::new(0x2C, "SRA H", operand!((Register::H)), vec![8]),
        Opcode::new(0x2D, "SRA L", operand!((Register::L)), vec![8]),
        Opcode::new(0x2E, "SRA [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x2F, "SRA A", operand!((Register::A)), vec![8]),
        Opcode::new(0x30, "SWAP B", operand!((Register::B)), vec![8]),
        Opcode::new(0x31, "SWAP C", operand!((Register::C)), vec![8]),
        Opcode::new(0x32, "SWAP D", operand!((Register::D)), vec![8]),
        Opcode::new(0x33, "SWAP E", operand!((Register::E)), vec![8]),
        Opcode::new(0x34, "SWAP H", operand!((Register::H)), vec![8]),
        Opcode::new(0x35, "SWAP L", operand!((Register::L)), vec![8]),
        Opcode::new(0x36, "SWAP [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x37, "SWAP A", operand!((Register::A)), vec![8]),
        Opcode::new(0x38, "SRL B", operand!((Register::B)), vec![8]),
        Opcode::new(0x39, "SRL C", operand!((Register::C)), vec![8]),
        Opcode::new(0x3A, "SRL D", operand!((Register::D)), vec![8]),
        Opcode::new(0x3B, "SRL E", operand!((Register::E)), vec![8]),
        Opcode::new(0x3C, "SRL H", operand!((Register::H)), vec![8]),
        Opcode::new(0x3D, "SRL L", operand!((Register::L)), vec![8]),
        Opcode::new(0x3E, "SRL [HL]", operand!([((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x3F, "SRL A", operand!((Register::A)), vec![8]),
        Opcode::new(0x40, "BIT 0, B", operand!((Literal(0)),(Register::B)), vec![8]),
        Opcode::new(0x41, "BIT 0, C", operand!((Literal(0)),(Register::C)), vec![8]),
        Opcode::new(0x42, "BIT 0, D", operand!((Literal(0)),(Register::D)), vec![8]),
        Opcode::new(0x43, "BIT 0, E", operand!((Literal(0)),(Register::E)), vec![8]),
        Opcode::new(0x44, "BIT 0, H", operand!((Literal(0)),(Register::H)), vec![8]),
        Opcode::new(0x45, "BIT 0, L", operand!((Literal(0)),(Register::L)), vec![8]),
        Opcode::new(0x46, "BIT 0, [HL]", operand!((Literal(0)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x47, "BIT 0, A", operand!((Literal(0)),(Register::A)), vec![8]),
        Opcode::new(0x48, "BIT 1, B", operand!((Literal(1)),(Register::B)), vec![8]),
        Opcode::new(0x49, "BIT 1, C", operand!((Literal(1)),(Register::C)), vec![8]),
        Opcode::new(0x4A, "BIT 1, D", operand!((Literal(1)),(Register::D)), vec![8]),
        Opcode::new(0x4B, "BIT 1, E", operand!((Literal(1)),(Register::E)), vec![8]),
        Opcode::new(0x4C, "BIT 1, H", operand!((Literal(1)),(Register::H)), vec![8]),
        Opcode::new(0x4D, "BIT 1, L", operand!((Literal(1)),(Register::L)), vec![8]),
        Opcode::new(0x4E, "BIT 1, [HL]", operand!((Literal(1)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x4F, "BIT 1, A", operand!((Literal(1)),(Register::A)), vec![8]),
        Opcode::new(0x50, "BIT 2, B", operand!((Literal(2)),(Register::B)), vec![8]),
        Opcode::new(0x51, "BIT 2, C", operand!((Literal(2)),(Register::C)), vec![8]),
        Opcode::new(0x52, "BIT 2, D", operand!((Literal(2)),(Register::D)), vec![8]),
        Opcode::new(0x53, "BIT 2, E", operand!((Literal(2)),(Register::E)), vec![8]),
        Opcode::new(0x54, "BIT 2, H", operand!((Literal(2)),(Register::H)), vec![8]),
        Opcode::new(0x55, "BIT 2, L", operand!((Literal(2)),(Register::L)), vec![8]),
        Opcode::new(0x56, "BIT 2, [HL]", operand!((Literal(2)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x57, "BIT 2, A", operand!((Literal(2)),(Register::A)), vec![8]),
        Opcode::new(0x58, "BIT 3, B", operand!((Literal(3)),(Register::B)), vec![8]),
        Opcode::new(0x59, "BIT 3, C", operand!((Literal(3)),(Register::C)), vec![8]),
        Opcode::new(0x5A, "BIT 3, D", operand!((Literal(3)),(Register::D)), vec![8]),
        Opcode::new(0x5B, "BIT 3, E", operand!((Literal(3)),(Register::E)), vec![8]),
        Opcode::new(0x5C, "BIT 3, H", operand!((Literal(3)),(Register::H)), vec![8]),
        Opcode::new(0x5D, "BIT 3, L", operand!((Literal(3)),(Register::L)), vec![8]),
        Opcode::new(0x5E, "BIT 3, [HL]", operand!((Literal(3)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x5F, "BIT 3, A", operand!((Literal(3)),(Register::A)), vec![8]),
        Opcode::new(0x60, "BIT 4, B", operand!((Literal(4)),(Register::B)), vec![8]),
        Opcode::new(0x61, "BIT 4, C", operand!((Literal(4)),(Register::C)), vec![8]),
        Opcode::new(0x62, "BIT 4, D", operand!((Literal(4)),(Register::D)), vec![8]),
        Opcode::new(0x63, "BIT 4, E", operand!((Literal(4)),(Register::E)), vec![8]),
        Opcode::new(0x64, "BIT 4, H", operand!((Literal(4)),(Register::H)), vec![8]),
        Opcode::new(0x65, "BIT 4, L", operand!((Literal(4)),(Register::L)), vec![8]),
        Opcode::new(0x66, "BIT 4, [HL]", operand!((Literal(4)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x67, "BIT 4, A", operand!((Literal(4)),(Register::A)), vec![8]),
        Opcode::new(0x68, "BIT 5, B", operand!((Literal(5)),(Register::B)), vec![8]),
        Opcode::new(0x69, "BIT 5, C", operand!((Literal(5)),(Register::C)), vec![8]),
        Opcode::new(0x6A, "BIT 5, D", operand!((Literal(5)),(Register::D)), vec![8]),
        Opcode::new(0x6B, "BIT 5, E", operand!((Literal(5)),(Register::E)), vec![8]),
        Opcode::new(0x6C, "BIT 5, H", operand!((Literal(5)),(Register::H)), vec![8]),
        Opcode::new(0x6D, "BIT 5, L", operand!((Literal(5)),(Register::L)), vec![8]),
        Opcode::new(0x6E, "BIT 5, [HL]", operand!((Literal(5)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x6F, "BIT 5, A", operand!((Literal(5)),(Register::A)), vec![8]),
        Opcode::new(0x70, "BIT 6, B", operand!((Literal(6)),(Register::B)), vec![8]),
        Opcode::new(0x71, "BIT 6, C", operand!((Literal(6)),(Register::C)), vec![8]),
        Opcode::new(0x72, "BIT 6, D", operand!((Literal(6)),(Register::D)), vec![8]),
        Opcode::new(0x73, "BIT 6, E", operand!((Literal(6)),(Register::E)), vec![8]),
        Opcode::new(0x74, "BIT 6, H", operand!((Literal(6)),(Register::H)), vec![8]),
        Opcode::new(0x75, "BIT 6, L", operand!((Literal(6)),(Register::L)), vec![8]),
        Opcode::new(0x76, "BIT 6, [HL]", operand!((Literal(6)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x77, "BIT 6, A", operand!((Literal(6)),(Register::A)), vec![8]),
        Opcode::new(0x78, "BIT 7, B", operand!((Literal(7)),(Register::B)), vec![8]),
        Opcode::new(0x79, "BIT 7, C", operand!((Literal(7)),(Register::C)), vec![8]),
        Opcode::new(0x7A, "BIT 7, D", operand!((Literal(7)),(Register::D)), vec![8]),
        Opcode::new(0x7B, "BIT 7, E", operand!((Literal(7)),(Register::E)), vec![8]),
        Opcode::new(0x7C, "BIT 7, H", operand!((Literal(7)),(Register::H)), vec![8]),
        Opcode::new(0x7D, "BIT 7, L", operand!((Literal(7)),(Register::L)), vec![8]),
        Opcode::new(0x7E, "BIT 7, [HL]", operand!((Literal(7)),[((Register::H),(Register::L))]), vec![12]),
        Opcode::new(0x7F, "BIT 7, A", operand!((Literal(7)),(Register::A)), vec![8]),
        Opcode::new(0x80, "RES 0, B", operand!((Literal(0)),(Register::B)), vec![8]),
        Opcode::new(0x81, "RES 0, C", operand!((Literal(0)),(Register::C)), vec![8]),
        Opcode::new(0x82, "RES 0, D", operand!((Literal(0)),(Register::D)), vec![8]),
        Opcode::new(0x83, "RES 0, E", operand!((Literal(0)),(Register::E)), vec![8]),
        Opcode::new(0x84, "RES 0, H", operand!((Literal(0)),(Register::H)), vec![8]),
        Opcode::new(0x85, "RES 0, L", operand!((Literal(0)),(Register::L)), vec![8]),
        Opcode::new(0x86, "RES 0, [HL]", operand!((Literal(0)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x87, "RES 0, A", operand!((Literal(0)),(Register::A)), vec![8]),
        Opcode::new(0x88, "RES 1, B", operand!((Literal(1)),(Register::B)), vec![8]),
        Opcode::new(0x89, "RES 1, C", operand!((Literal(1)),(Register::C)), vec![8]),
        Opcode::new(0x8A, "RES 1, D", operand!((Literal(1)),(Register::D)), vec![8]),
        Opcode::new(0x8B, "RES 1, E", operand!((Literal(1)),(Register::E)), vec![8]),
        Opcode::new(0x8C, "RES 1, H", operand!((Literal(1)),(Register::H)), vec![8]),
        Opcode::new(0x8D, "RES 1, L", operand!((Literal(1)),(Register::L)), vec![8]),
        Opcode::new(0x8E, "RES 1, [HL]", operand!((Literal(1)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x8F, "RES 1, A", operand!((Literal(1)),(Register::A)), vec![8]),
        Opcode::new(0x90, "RES 2, B", operand!((Literal(2)),(Register::B)), vec![8]),
        Opcode::new(0x91, "RES 2, C", operand!((Literal(2)),(Register::C)), vec![8]),
        Opcode::new(0x92, "RES 2, D", operand!((Literal(2)),(Register::D)), vec![8]),
        Opcode::new(0x93, "RES 2, E", operand!((Literal(2)),(Register::E)), vec![8]),
        Opcode::new(0x94, "RES 2, H", operand!((Literal(2)),(Register::H)), vec![8]),
        Opcode::new(0x95, "RES 2, L", operand!((Literal(2)),(Register::L)), vec![8]),
        Opcode::new(0x96, "RES 2, [HL]", operand!((Literal(2)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x97, "RES 2, A", operand!((Literal(2)),(Register::A)), vec![8]),
        Opcode::new(0x98, "RES 3, B", operand!((Literal(3)),(Register::B)), vec![8]),
        Opcode::new(0x99, "RES 3, C", operand!((Literal(3)),(Register::C)), vec![8]),
        Opcode::new(0x9A, "RES 3, D", operand!((Literal(3)),(Register::D)), vec![8]),
        Opcode::new(0x9B, "RES 3, E", operand!((Literal(3)),(Register::E)), vec![8]),
        Opcode::new(0x9C, "RES 3, H", operand!((Literal(3)),(Register::H)), vec![8]),
        Opcode::new(0x9D, "RES 3, L", operand!((Literal(3)),(Register::L)), vec![8]),
        Opcode::new(0x9E, "RES 3, [HL]", operand!((Literal(3)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0x9F, "RES 3, A", operand!((Literal(3)),(Register::A)), vec![8]),
        Opcode::new(0xA0, "RES 4, B", operand!((Literal(4)),(Register::B)), vec![8]),
        Opcode::new(0xA1, "RES 4, C", operand!((Literal(4)),(Register::C)), vec![8]),
        Opcode::new(0xA2, "RES 4, D", operand!((Literal(4)),(Register::D)), vec![8]),
        Opcode::new(0xA3, "RES 4, E", operand!((Literal(4)),(Register::E)), vec![8]),
        Opcode::new(0xA4, "RES 4, H", operand!((Literal(4)),(Register::H)), vec![8]),
        Opcode::new(0xA5, "RES 4, L", operand!((Literal(4)),(Register::L)), vec![8]),
        Opcode::new(0xA6, "RES 4, [HL]", operand!((Literal(4)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xA7, "RES 4, A", operand!((Literal(4)),(Register::A)), vec![8]),
        Opcode::new(0xA8, "RES 5, B", operand!((Literal(5)),(Register::B)), vec![8]),
        Opcode::new(0xA9, "RES 5, C", operand!((Literal(5)),(Register::C)), vec![8]),
        Opcode::new(0xAA, "RES 5, D", operand!((Literal(5)),(Register::D)), vec![8]),
        Opcode::new(0xAB, "RES 5, E", operand!((Literal(5)),(Register::E)), vec![8]),
        Opcode::new(0xAC, "RES 5, H", operand!((Literal(5)),(Register::H)), vec![8]),
        Opcode::new(0xAD, "RES 5, L", operand!((Literal(5)),(Register::L)), vec![8]),
        Opcode::new(0xAE, "RES 5, [HL]", operand!((Literal(5)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xAF, "RES 5, A", operand!((Literal(5)),(Register::A)), vec![8]),
        Opcode::new(0xB0, "RES 6, B", operand!((Literal(6)),(Register::B)), vec![8]),
        Opcode::new(0xB1, "RES 6, C", operand!((Literal(6)),(Register::C)), vec![8]),
        Opcode::new(0xB2, "RES 6, D", operand!((Literal(6)),(Register::D)), vec![8]),
        Opcode::new(0xB3, "RES 6, E", operand!((Literal(6)),(Register::E)), vec![8]),
        Opcode::new(0xB4, "RES 6, H", operand!((Literal(6)),(Register::H)), vec![8]),
        Opcode::new(0xB5, "RES 6, L", operand!((Literal(6)),(Register::L)), vec![8]),
        Opcode::new(0xB6, "RES 6, [HL]", operand!((Literal(6)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xB7, "RES 6, A", operand!((Literal(6)),(Register::A)), vec![8]),
        Opcode::new(0xB8, "RES 7, B", operand!((Literal(7)),(Register::B)), vec![8]),
        Opcode::new(0xB9, "RES 7, C", operand!((Literal(7)),(Register::C)), vec![8]),
        Opcode::new(0xBA, "RES 7, D", operand!((Literal(7)),(Register::D)), vec![8]),
        Opcode::new(0xBB, "RES 7, E", operand!((Literal(7)),(Register::E)), vec![8]),
        Opcode::new(0xBC, "RES 7, H", operand!((Literal(7)),(Register::H)), vec![8]),
        Opcode::new(0xBD, "RES 7, L", operand!((Literal(7)),(Register::L)), vec![8]),
        Opcode::new(0xBE, "RES 7, [HL]", operand!((Literal(7)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xBF, "RES 7, A", operand!((Literal(7)),(Register::A)), vec![8]),
        Opcode::new(0xC0, "SET 0, B", operand!((Literal(0)),(Register::B)), vec![8]),
        Opcode::new(0xC1, "SET 0, C", operand!((Literal(0)),(Register::C)), vec![8]),
        Opcode::new(0xC2, "SET 0, D", operand!((Literal(0)),(Register::D)), vec![8]),
        Opcode::new(0xC3, "SET 0, E", operand!((Literal(0)),(Register::E)), vec![8]),
        Opcode::new(0xC4, "SET 0, H", operand!((Literal(0)),(Register::H)), vec![8]),
        Opcode::new(0xC5, "SET 0, L", operand!((Literal(0)),(Register::L)), vec![8]),
        Opcode::new(0xC6, "SET 0, [HL]", operand!((Literal(0)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xC7, "SET 0, A", operand!((Literal(0)),(Register::A)), vec![8]),
        Opcode::new(0xC8, "SET 1, B", operand!((Literal(1)),(Register::B)), vec![8]),
        Opcode::new(0xC9, "SET 1, C", operand!((Literal(1)),(Register::C)), vec![8]),
        Opcode::new(0xCA, "SET 1, D", operand!((Literal(1)),(Register::D)), vec![8]),
        Opcode::new(0xCB, "SET 1, E", operand!((Literal(1)),(Register::E)), vec![8]),
        Opcode::new(0xCC, "SET 1, H", operand!((Literal(1)),(Register::H)), vec![8]),
        Opcode::new(0xCD, "SET 1, L", operand!((Literal(1)),(Register::L)), vec![8]),
        Opcode::new(0xCE, "SET 1, [HL]", operand!((Literal(1)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xCF, "SET 1, A", operand!((Literal(1)),(Register::A)), vec![8]),
        Opcode::new(0xD0, "SET 2, B", operand!((Literal(2)),(Register::B)), vec![8]),
        Opcode::new(0xD1, "SET 2, C", operand!((Literal(2)),(Register::C)), vec![8]),
        Opcode::new(0xD2, "SET 2, D", operand!((Literal(2)),(Register::D)), vec![8]),
        Opcode::new(0xD3, "SET 2, E", operand!((Literal(2)),(Register::E)), vec![8]),
        Opcode::new(0xD4, "SET 2, H", operand!((Literal(2)),(Register::H)), vec![8]),
        Opcode::new(0xD5, "SET 2, L", operand!((Literal(2)),(Register::L)), vec![8]),
        Opcode::new(0xD6, "SET 2, [HL]", operand!((Literal(2)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xD7, "SET 2, A", operand!((Literal(2)),(Register::A)), vec![8]),
        Opcode::new(0xD8, "SET 3, B", operand!((Literal(3)),(Register::B)), vec![8]),
        Opcode::new(0xD9, "SET 3, C", operand!((Literal(3)),(Register::C)), vec![8]),
        Opcode::new(0xDA, "SET 3, D", operand!((Literal(3)),(Register::D)), vec![8]),
        Opcode::new(0xDB, "SET 3, E", operand!((Literal(3)),(Register::E)), vec![8]),
        Opcode::new(0xDC, "SET 3, H", operand!((Literal(3)),(Register::H)), vec![8]),
        Opcode::new(0xDD, "SET 3, L", operand!((Literal(3)),(Register::L)), vec![8]),
        Opcode::new(0xDE, "SET 3, [HL]", operand!((Literal(3)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xDF, "SET 3, A", operand!((Literal(3)),(Register::A)), vec![8]),
        Opcode::new(0xE0, "SET 4, B", operand!((Literal(4)),(Register::B)), vec![8]),
        Opcode::new(0xE1, "SET 4, C", operand!((Literal(4)),(Register::C)), vec![8]),
        Opcode::new(0xE2, "SET 4, D", operand!((Literal(4)),(Register::D)), vec![8]),
        Opcode::new(0xE3, "SET 4, E", operand!((Literal(4)),(Register::E)), vec![8]),
        Opcode::new(0xE4, "SET 4, H", operand!((Literal(4)),(Register::H)), vec![8]),
        Opcode::new(0xE5, "SET 4, L", operand!((Literal(4)),(Register::L)), vec![8]),
        Opcode::new(0xE6, "SET 4, [HL]", operand!((Literal(4)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xE7, "SET 4, A", operand!((Literal(4)),(Register::A)), vec![8]),
        Opcode::new(0xE8, "SET 5, B", operand!((Literal(5)),(Register::B)), vec![8]),
        Opcode::new(0xE9, "SET 5, C", operand!((Literal(5)),(Register::C)), vec![8]),
        Opcode::new(0xEA, "SET 5, D", operand!((Literal(5)),(Register::D)), vec![8]),
        Opcode::new(0xEB, "SET 5, E", operand!((Literal(5)),(Register::E)), vec![8]),
        Opcode::new(0xEC, "SET 5, H", operand!((Literal(5)),(Register::H)), vec![8]),
        Opcode::new(0xED, "SET 5, L", operand!((Literal(5)),(Register::L)), vec![8]),
        Opcode::new(0xEE, "SET 5, [HL]", operand!((Literal(5)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xEF, "SET 5, A", operand!((Literal(5)),(Register::A)), vec![8]),
        Opcode::new(0xF0, "SET 6, B", operand!((Literal(6)),(Register::B)), vec![8]),
        Opcode::new(0xF1, "SET 6, C", operand!((Literal(6)),(Register::C)), vec![8]),
        Opcode::new(0xF2, "SET 6, D", operand!((Literal(6)),(Register::D)), vec![8]),
        Opcode::new(0xF3, "SET 6, E", operand!((Literal(6)),(Register::E)), vec![8]),
        Opcode::new(0xF4, "SET 6, H", operand!((Literal(6)),(Register::H)), vec![8]),
        Opcode::new(0xF5, "SET 6, L", operand!((Literal(6)),(Register::L)), vec![8]),
        Opcode::new(0xF6, "SET 6, [HL]", operand!((Literal(6)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xF7, "SET 6, A", operand!((Literal(6)),(Register::A)), vec![8]),
        Opcode::new(0xF8, "SET 7, B", operand!((Literal(7)),(Register::B)), vec![8]),
        Opcode::new(0xF9, "SET 7, C", operand!((Literal(7)),(Register::C)), vec![8]),
        Opcode::new(0xFA, "SET 7, D", operand!((Literal(7)),(Register::D)), vec![8]),
        Opcode::new(0xFB, "SET 7, E", operand!((Literal(7)),(Register::E)), vec![8]),
        Opcode::new(0xFC, "SET 7, H", operand!((Literal(7)),(Register::H)), vec![8]),
        Opcode::new(0xFD, "SET 7, L", operand!((Literal(7)),(Register::L)), vec![8]),
        Opcode::new(0xFE, "SET 7, [HL]", operand!((Literal(7)),[((Register::H),(Register::L))]), vec![16]),
        Opcode::new(0xFF, "SET 7, A", operand!((Literal(7)),(Register::A)), vec![8])
    ]
}