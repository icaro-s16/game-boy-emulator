use core::panic;
use std::{any::Any, unreachable};

use crate::core::{
    cpu::*, 
    memory::Memory
};

#[derive(Clone)]
pub struct Instruction{
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub operands: OperandSet,
    pub cycles: Vec<u8>,
    pub execute: &'static dyn Fn(&Instruction, &mut Cpu, &mut Memory)
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
pub struct Literal(pub usize);

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
pub struct Address(pub Box<dyn Operand>);

pub trait Operand{
    fn as_any(&self) -> &dyn Any;
    
    fn clone_box(&self) -> Box<dyn Operand>; 

    fn get_type(&self) -> OperandType;

    /* NOTE: Every operand value can be interpreted 
     * as an usize, for the instructions we just have 
     * to know the bits of the value and not its type
     */
    fn get_value(
        &self, 
        cpu: &mut Cpu,
        memory: &Memory
    ) -> usize;
    
    fn update(
        &self, 
        _value: usize, 
        _cpu: &mut Cpu, 
        _memory: &mut Memory
    ){
        panic!("[ERROR] The operand can not be updated")
    }
}

macro_rules! operand_default_methods {
    () => {
        fn as_any(&self) -> &dyn Any{
            self
        }

        fn clone_box(&self) -> Box<dyn Operand>{
            Box::new(self.clone())
        }
    };
}

impl Instruction{
    pub fn new(
        opcode: u8, 
        mnemonic: &'static str, 
        operands: OperandSet, 
        cycles: Vec<u8>,
        execute: &'static dyn Fn(&Instruction, &mut Cpu, &mut Memory)
    ) -> Self{
        Self {opcode, mnemonic, operands, cycles, execute}
    }

    pub fn get_t_cycles(
        &self, 
        cpu: &mut Cpu, 
        memory: &mut Memory
    ) -> u8{
        if 
            self.cycles.len() == 2
        {
            match self.operands {
                OperandSet::Binary(ref op, _) => {
                    if op.get_value(cpu, memory) != 0 {self.cycles[1]}
                    else {self.cycles[0]}
                },
                OperandSet::Unary(ref op) => {
                    if op.get_value(cpu, memory) != 0 {self.cycles[1]}
                    else {self.cycles[0]}
                },
                OperandSet::Nullary => self.cycles[0] 
            }
        }
        else {
            self.cycles[0]
        }
    }
}


impl Clone for Box<dyn Operand>{
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Operand for Address{
    operand_default_methods!();

    fn get_type(&self) -> OperandType{
        OperandType::Address
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu,
        memory: &Memory
    ) -> usize{
        if 
            self.0.get_type() == OperandType::Address   ||
            self.0.get_type() == OperandType::Flag      ||
            self.0.get_type() == OperandType::Literal
        {
            panic!("[ERROR] The value inside of an address can not be an address, a flag or a literal");
        }

        let address  = self.0.get_value(cpu, memory);
        memory[
            address
        ] as usize
    }

    fn update(
        &self, 
        value: usize, 
        cpu: &mut Cpu, 
        memory: &mut Memory
    ){
        let address = self.0.get_value(cpu, memory);

        if 
            address == DIV_ADDR 
        {
            memory[DIV_ADDR] = 0;
        } 
        else{ 
            memory[address] = value as u8;
        }
    }
}

impl Address{
    pub fn get_value_with_offset(
        &self, 
        offset: usize, 
        cpu: &mut Cpu, 
        memory: &Memory
    ) -> usize{
        if 
            self.0.get_type() == OperandType::Address   ||
            self.0.get_type() == OperandType::Flag      ||
            self.0.get_type() == OperandType::Literal
        {
            panic!("[ERROR] The value inside of an address can not be an address, a flag or a literal");
        }

        let address = self.0.get_value(cpu, memory) + offset;
        memory[
            address
        ] as usize
    }

    pub fn update_with_offset(
        &self, 
        value: usize, 
        offset: usize, 
        cpu: &mut Cpu, 
        memory: &mut Memory
    ){
        let address = self.0.get_value(cpu, memory) + offset;
        
        if 
            address + offset == DIV_ADDR 
        {
            memory[DIV_ADDR] = 0;
        } 
        else{ 
            memory[address] = value as u8;
        }
    }
}

impl Operand for I8{
    operand_default_methods!();

    fn get_type(&self) -> OperandType{
        OperandType::I8
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu, 
        memory: &Memory
    ) -> usize{
        let value = memory[cpu.pc as usize].clone() as usize;
        cpu.pc += 1;
        value
    }
}

impl Operand for U8{
    operand_default_methods!();

    fn get_type(&self) -> OperandType{
        OperandType::U8
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu,
        memory: &Memory
    ) -> usize{
        let value = memory[cpu.pc as usize].clone() as usize;
        cpu.pc += 1;
        value
    }
}

impl Operand for U16{
    operand_default_methods!();
    
    fn get_type(&self) -> OperandType{
        OperandType::U16
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu, 
        memory: &Memory
    ) -> usize{
        let v1 = memory[cpu.pc as usize].clone();
        cpu.pc += 1;
        let v2 = memory[cpu.pc as usize].clone();
        cpu.pc += 1;

        /* NOTE: Game Boy use little endian. */
        i16::from_be_bytes([v2, v1]) as usize
    }
}

impl Operand for Literal{
    operand_default_methods!();
    
    fn get_type(&self) -> OperandType{
        OperandType::Literal
    }

    fn get_value(
        &self, 
        _cpu: &mut Cpu, 
        _memory: &Memory
    ) -> usize{
        self.0 as usize
    }
}

impl Operand for Flag{
    operand_default_methods!();

    fn get_type(&self) -> OperandType{
        OperandType::Flag
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu, 
        _memory: &Memory
    ) -> usize{
        match self{
            Flag::Z => {
                cpu.get_zflag() as usize
            },
            Flag::NZ => {
                !cpu.get_zflag() as usize
            }
            Flag::C => {
                cpu.get_cflag() as usize
            },
            Flag::NC => {
                !cpu.get_cflag() as usize
            }
            Flag::N => {
                cpu.get_zflag() as usize
            },
            Flag::H => {
                cpu.get_zflag() as usize
            }
        }
    }
}

impl Operand for Register{
    operand_default_methods!();
    
    fn get_type(&self) -> OperandType{
        OperandType::Register
    }

    fn get_value(
        &self, 
        cpu: &mut Cpu, 
        _memory: &Memory
    ) -> usize{
        match self{
            Register::Union(r1, r2) => {
                let hbyte = match r1.as_ref(){
                    Register::A => cpu.a.clone(),
                    Register::B => cpu.b.clone(),
                    Register::C => cpu.c.clone(),
                    Register::D => cpu.d.clone(),
                    Register::E => cpu.e.clone(),
                    Register::F => cpu.f.clone(),
                    Register::H => cpu.h.clone(),
                    Register::L => cpu.l.clone(),
                    _ => panic!("[ERROR] An union register can not be an recursive type or the PC/SP")
                };
                let lbyte = match r2.as_ref(){
                    Register::A => cpu.a.clone(),
                    Register::B => cpu.b.clone(),
                    Register::C => cpu.c.clone(),
                    Register::D => cpu.d.clone(),
                    Register::E => cpu.e.clone(),
                    Register::F => cpu.f.clone(),
                    Register::H => cpu.h.clone(),
                    Register::L => cpu.l.clone(),
                    _ => panic!("[ERROR] An union register can not be an recursive type or the PC/SP")
                };
                i16::from_be_bytes([hbyte, lbyte]) as usize
            },
            Register::A => {cpu.a as usize},
            Register::B => {cpu.b as usize},
            Register::C => {cpu.c as usize},
            Register::D => {cpu.d as usize},
            Register::E => {cpu.e as usize},
            Register::F => {cpu.f as usize},
            Register::H => {cpu.h as usize},
            Register::L => {cpu.l as usize},
            _ => panic!("[ERROR] SP and PC can not be an operand register")
        }
    }

    fn update(
        &self, 
        value: usize, 
        cpu: &mut Cpu, 
        _memory: &mut Memory
    ){
        let word = value as u16;
        let bytes = word.to_be_bytes();
        match self {
            Register::Union(r1, r2) => {
                match r1.as_ref(){
                    Register::A => cpu.a = bytes[0],
                    Register::B => cpu.b = bytes[0],
                    Register::C => cpu.c = bytes[0],
                    Register::D => cpu.d = bytes[0],
                    Register::E => cpu.e = bytes[0],
                    Register::F => cpu.f = bytes[0],
                    Register::H => cpu.h = bytes[0],
                    Register::L => cpu.l = bytes[0],
                    _ => {panic!("[ERROR] An union register can not have another union or the PC/SP register")}
                }
                match r2.as_ref(){
                    Register::A => cpu.a = bytes[1],
                    Register::B => cpu.b = bytes[1],
                    Register::C => cpu.c = bytes[1],
                    Register::D => cpu.d = bytes[1],
                    Register::E => cpu.e = bytes[1],
                    Register::F => cpu.f = bytes[1],
                    Register::H => cpu.h = bytes[1],
                    Register::L => cpu.l = bytes[1],
                    _ => {panic!("[ERROR] An union register can not have another union or the PC/SP register")}
                }
            },
            Register::A => cpu.a = bytes[1],
            Register::B => cpu.b = bytes[1],
            Register::C => cpu.c = bytes[1],
            Register::D => cpu.d = bytes[1],
            Register::E => cpu.e = bytes[1],
            Register::F => cpu.f = bytes[1],
            Register::H => cpu.h = bytes[1],
            Register::L => cpu.l = bytes[1],
            Register::SP => cpu.sp = word,
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

