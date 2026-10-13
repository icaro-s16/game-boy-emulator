use crate::core::system::{
    Register,
    System
};

#[derive(Clone)]
pub struct Opcode{
    pub mnemonic: &'static str,
    pub operand: OperandSet,
    pub cycles: u16
}
pub trait Operand{
    /* Source - https://stackoverflow.com/a/65204236
     * Posted by Freyja, modified by community. See post 'Timeline' for change history
     * Retrieved 2026-10-07, License - CC BY-SA 4.0
     */
    fn clone_box(&self) -> Box<dyn Operand>; 

    fn get_operand_type(&self) -> OperandType;

    /* NOTE: Every operand value can be interpreted 
     * as an usize, and after that can be casted to a
     * differente type, this allow us to use any
     * operand as a index even when we use dyn Trait.
     */
    fn get_operand_value(&self, system: &mut System) -> usize;

    fn update_system(&self, value: usize, system: &mut System);
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum OperandType{
    Address,
    URegister,
    Register,
    U16,
    U8,
    I8
}

#[derive(Clone)]
pub struct Address(Box<dyn Operand>);

impl Opcode{
    pub fn new(mnemonic: &'static str, operand: OperandSet, cycles: u16) -> Self{
        Self { mnemonic, operand, cycles }
    }
}

impl Clone for Box<dyn Operand>{
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl Operand for Address{
    fn clone_box(&self) -> Box<dyn Operand>{
        Box::new(self.clone())
    }

    fn get_operand_type(&self) -> OperandType{
        OperandType::Address
    }

    fn get_operand_value(&self, system: &mut System) -> usize{
        if self.0.get_operand_type() == OperandType::Address { 
            panic!("[ERROR] Invalid operand type address(address(...)) "); 
        }
        
        system.vram[
            self.0.get_operand_value(system)
        ] as usize
    }

    fn update_system(&self, value: usize, system: &mut System){
        system.vram[self.0.get_operand_value(system)] = value as u8;
    }
}

impl Operand for I8{
    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }

    fn get_operand_type(&self) -> OperandType{
        OperandType::I8
    }

    fn get_operand_value(&self, system: &mut System) -> usize{
        let value = system.vram[system.pc as usize].clone() as usize;
        system.pc += 1;
        value
    }

    fn update_system(&self, _value: usize, _system: &mut System){
        panic!("[ERROR] System can not be updated by an literal i8");
    }
}

impl Operand for U8{
    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }

    fn get_operand_type(&self) -> OperandType{
        OperandType::U8
    }

    fn get_operand_value(&self, system: &mut System) -> usize{
        let value = system.vram[system.pc as usize].clone() as usize;
        system.pc += 1;
        value
    }

    fn update_system(&self, _value: usize, _system: &mut System){
        panic!("[ERROR] System can not be updated by an literal u8");
    }
}

impl Operand for U16{
    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
    
    fn get_operand_type(&self) -> OperandType{
        OperandType::U16
    }

    fn get_operand_value(&self, system: &mut System) -> usize{
        let v1 = system.vram[system.pc as usize].clone();
        system.pc += 1;
        let v2 = system.vram[system.pc as usize].clone();
        system.pc += 1;

        /* NOTE: Game Boy use little endian 
         */
        i16::from_be_bytes([v2, v1]) as usize
    }

    fn update_system(&self, _value: usize, _system: &mut System){
        panic!("[ERROR] System can not be updated by an literal u16");
    }
}

impl Operand for Register{
    fn clone_box(&self) -> Box<dyn Operand> {
        Box::new(self.clone())
    }
    
    fn get_operand_type(&self) -> OperandType{
        if let Register::Union(_, _) = self {
            OperandType::URegister
        }
        else {
            OperandType::Register
        }
    }

    fn get_operand_value(&self, system: &mut System) -> usize{
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

    fn update_system(&self, value: usize, system: &mut System){
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
                    _ => {panic!("[ERROR] An union register can not be an recursive type or the PC/SP")}
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
                    _ => {panic!("[ERROR] An union register can not be an recursive type or the PC/SP")}
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
        Some(Opcode::new("NOP", operand!(), 4)),
        Some(Opcode::new("LD BC, d16", operand!((Register::B, Register::C), U16), 12)),
        Some(Opcode::new("LD [BC], A", operand!([(Register::B, Register::C)], (Register::A)), 8)),
        Some(Opcode::new("INC BC", operand!((Register::B, Register::C)), 8)),
        Some(Opcode::new("INC B", operand!((Register::B)), 4)),
        Some(Opcode::new("DEC B", operand!((Register::B)), 4)),
        Some(Opcode::new("LD B, d8", operand!((Register::B), U8), 8)),
        Some(Opcode::new("RLCA", operand!((Register::A)), 4)),
        Some(Opcode::new("LD [a16], SP", operand!([U16], (Register::SP)), 20)),
        Some(Opcode::new("ADD HL, BC", operand!((Register::H, Register::L), (Register::B, Register::C)), 8)),
        Some(Opcode::new("LD A, [BC]", operand!((Register::A), [(Register::B, Register::C)]), 8)),
        Some(Opcode::new("DEC BC", operand!((Register::B, Register::C)), 8)),
        Some(Opcode::new("INC C", operand!((Register::C)), 4)),
        Some(Opcode::new("DEC C", operand!((Register::C)), 4)),
        Some(Opcode::new("LD C, d8", operand!((Register::C), U8), 8)),
        Some(Opcode::new("RRCA", operand!((Register::A)), 4)),
        Some(Opcode::new("STOP 0", operand!(U8), 4)),
        Some(Opcode::new("LD DE, d16", operand!((Register::D, Register::E), U16), 12)),
        Some(Opcode::new("LD [DE], A", operand!([(Register::D, Register::E)], (Register::A)), 8)),
        Some(Opcode::new("INC DE", operand!((Register::D, Register::E)), 8)),
        Some(Opcode::new("INC D", operand!((Register::D)), 4)),
        Some(Opcode::new("DEC D", operand!((Register::D)), 4)),
        Some(Opcode::new("LD D, d8", operand!((Register::D), U8), 8)),
        Some(Opcode::new("RLA", operand!((Register::A)), 4)),
        Some(Opcode::new("JR r8", operand!(I8), 12)),
        Some(Opcode::new("ADD HL, DE", operand!((Register::H, Register::L), (Register::D, Register::E)), 8)),
        Some(Opcode::new("LD A, [DE]", operand!((Register::A), [(Register::D, Register::E)]), 8)),
        Some(Opcode::new("DEC DE", operand!((Register::D, Register::E)), 8)),
        Some(Opcode::new("INC E", operand!((Register::E)), 4)),
        Some(Opcode::new("DEC E", operand!((Register::E)), 4)),
        Some(Opcode::new("LD E, d8", operand!((Register::E), U8), 8)),
        Some(Opcode::new("RRA", operand!((Register::A)), 4)),
        Some(Opcode::new("JR NZ, r8", operand!(I8), 12)),
        Some(Opcode::new("LD HL, d16", operand!((Register::H, Register::L), U16), 12)),
        Some(Opcode::new("LD [HL+], A", operand!([(Register::H, Register::L)], (Register::A)), 8)),
        Some(Opcode::new("INC HL", operand!((Register::H, Register::L)), 8)),
        Some(Opcode::new("INC H", operand!((Register::H)), 4)),
        Some(Opcode::new("DEC H", operand!((Register::H)), 4)),
        Some(Opcode::new("LD H, d8", operand!((Register::H), U8), 8)),
        Some(Opcode::new("DAA", operand!(), 4)),
        Some(Opcode::new("JR Z, r8", operand!(I8), 12)),
        Some(Opcode::new("ADD HL, HL", operand!((Register::H, Register::L), (Register::H, Register::L)), 8)),
        Some(Opcode::new("LD A, [HL+]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("DEC HL", operand!((Register::H, Register::L)), 8)),
        Some(Opcode::new("INC L", operand!((Register::L)), 4)),
        Some(Opcode::new("DEC L", operand!((Register::L)), 4)),
        Some(Opcode::new("LD L, d8", operand!((Register::L), U8), 8)),
        Some(Opcode::new("CPL", operand!(), 4)),
        Some(Opcode::new("JR NC, r8", operand!(I8), 12)),
        Some(Opcode::new("LD SP, d16", operand!((Register::SP), U16), 12)),
        Some(Opcode::new("LD [HL-], A", operand!([(Register::H, Register::L)], (Register::A)), 8)),
        Some(Opcode::new("INC SP", operand!((Register::SP)), 8)),
        Some(Opcode::new("INC [HL]", operand!([(Register::H, Register::L)]), 12)),
        Some(Opcode::new("DEC [HL]", operand!([(Register::H, Register::L)]), 12)),
        Some(Opcode::new("LD [HL], d8", operand!([(Register::H, Register::L)], U8), 12)),
        Some(Opcode::new("SCF", operand!(), 4)),
        Some(Opcode::new("JR C, r8", operand!(I8), 12)),
        Some(Opcode::new("ADD HL, SP", operand!((Register::H, Register::L), (Register::SP)), 8)),
        Some(Opcode::new("LD A, [HL-]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("DEC SP", operand!((Register::SP)), 8)),
        Some(Opcode::new("INC A", operand!((Register::A)), 4)),
        Some(Opcode::new("DEC A", operand!((Register::A)), 4)),
        Some(Opcode::new("LD A, d8", operand!((Register::A), U8), 8)),
        Some(Opcode::new("CCF", operand!(), 4)),
        Some(Opcode::new("LD B, B", operand!((Register::B), (Register::B)), 4)),
        Some(Opcode::new("LD B, C", operand!((Register::B), (Register::C)), 4)),
        Some(Opcode::new("LD B, D", operand!((Register::B), (Register::D)), 4)),
        Some(Opcode::new("LD B, E", operand!((Register::B), (Register::E)), 4)),
        Some(Opcode::new("LD B, H", operand!((Register::B), (Register::H)), 4)),
        Some(Opcode::new("LD B, L", operand!((Register::B), (Register::L)), 4)),
        Some(Opcode::new("LD B, [HL]", operand!((Register::B), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD B, A", operand!((Register::B), (Register::A)), 4)),
        Some(Opcode::new("LD C, B", operand!((Register::C), (Register::B)), 4)),
        Some(Opcode::new("LD C, C", operand!((Register::C), (Register::C)), 4)),
        Some(Opcode::new("LD C, D", operand!((Register::C), (Register::D)), 4)),
        Some(Opcode::new("LD C, E", operand!((Register::C), (Register::E)), 4)),
        Some(Opcode::new("LD C, H", operand!((Register::C), (Register::H)), 4)),
        Some(Opcode::new("LD C, L", operand!((Register::C), (Register::L)), 4)),
        Some(Opcode::new("LD C, [HL]", operand!((Register::C), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD C, A", operand!((Register::C), (Register::A)), 4)),
        Some(Opcode::new("LD D, B", operand!((Register::D), (Register::B)), 4)),
        Some(Opcode::new("LD D, C", operand!((Register::D), (Register::C)), 4)),
        Some(Opcode::new("LD D, D", operand!((Register::D), (Register::D)), 4)),
        Some(Opcode::new("LD D, E", operand!((Register::D), (Register::E)), 4)),
        Some(Opcode::new("LD D, H", operand!((Register::D), (Register::H)), 4)),
        Some(Opcode::new("LD D, L", operand!((Register::D), (Register::L)), 4)),
        Some(Opcode::new("LD D, [HL]", operand!((Register::D), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD D, A", operand!((Register::D), (Register::A)), 4)),
        Some(Opcode::new("LD E, B", operand!((Register::E), (Register::B)), 4)),
        Some(Opcode::new("LD E, C", operand!((Register::E), (Register::C)), 4)),
        Some(Opcode::new("LD E, D", operand!((Register::E), (Register::D)), 4)),
        Some(Opcode::new("LD E, E", operand!((Register::E), (Register::E)), 4)),
        Some(Opcode::new("LD E, H", operand!((Register::E), (Register::H)), 4)),
        Some(Opcode::new("LD E, L", operand!((Register::E), (Register::L)), 4)),
        Some(Opcode::new("LD E, [HL]", operand!((Register::E), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD E, A", operand!((Register::E), (Register::A)), 4)),
        Some(Opcode::new("LD H, B", operand!((Register::H), (Register::B)), 4)),
        Some(Opcode::new("LD H, C", operand!((Register::H), (Register::C)), 4)),
        Some(Opcode::new("LD H, D", operand!((Register::H), (Register::D)), 4)),
        Some(Opcode::new("LD H, E", operand!((Register::H), (Register::E)), 4)),
        Some(Opcode::new("LD H, H", operand!((Register::H), (Register::H)), 4)),
        Some(Opcode::new("LD H, L", operand!((Register::H), (Register::L)), 4)),
        Some(Opcode::new("LD H, [HL]", operand!((Register::H), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD H, A", operand!((Register::H), (Register::A)), 4)),
        Some(Opcode::new("LD L, B", operand!((Register::L), (Register::B)), 4)),
        Some(Opcode::new("LD L, C", operand!((Register::L), (Register::C)), 4)),
        Some(Opcode::new("LD L, D", operand!((Register::L), (Register::D)), 4)),
        Some(Opcode::new("LD L, E", operand!((Register::L), (Register::E)), 4)),
        Some(Opcode::new("LD L, H", operand!((Register::L), (Register::H)), 4)),
        Some(Opcode::new("LD L, L", operand!((Register::L), (Register::L)), 4)),
        Some(Opcode::new("LD L, [HL]", operand!((Register::L), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD L, A", operand!((Register::L), (Register::A)), 4)),
        Some(Opcode::new("LD [HL], B", operand!([(Register::H, Register::L)], (Register::B)), 8)),
        Some(Opcode::new("LD [HL], C", operand!([(Register::H, Register::L)], (Register::C)), 8)),
        Some(Opcode::new("LD [HL], D", operand!([(Register::H, Register::L)], (Register::D)), 8)),
        Some(Opcode::new("LD [HL], E", operand!([(Register::H, Register::L)], (Register::E)), 8)),
        Some(Opcode::new("LD [HL], H", operand!([(Register::H, Register::L)], (Register::H)), 8)),
        Some(Opcode::new("LD [HL], L", operand!([(Register::H, Register::L)], (Register::L)), 8)),
        Some(Opcode::new("HALT", operand!(), 4)),
        Some(Opcode::new("LD [HL], A", operand!([(Register::H, Register::L)], (Register::A)), 8)),
        Some(Opcode::new("LD A, B", operand!((Register::A), (Register::B)), 4)),
        Some(Opcode::new("LD A, C", operand!((Register::A), (Register::C)), 4)),
        Some(Opcode::new("LD A, D", operand!((Register::A), (Register::D)), 4)),
        Some(Opcode::new("LD A, E", operand!((Register::A), (Register::E)), 4)),
        Some(Opcode::new("LD A, H", operand!((Register::A), (Register::H)), 4)),
        Some(Opcode::new("LD A, L", operand!((Register::A), (Register::L)), 4)),
        Some(Opcode::new("LD A, [HL]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("LD A, A", operand!((Register::A), (Register::A)), 4)),
        Some(Opcode::new("ADD A, B", operand!((Register::A), (Register::B)), 4)),
        Some(Opcode::new("ADD A, C", operand!((Register::A), (Register::C)), 4)),
        Some(Opcode::new("ADD A, D", operand!((Register::A), (Register::D)), 4)),
        Some(Opcode::new("ADD A, E", operand!((Register::A), (Register::E)), 4)),
        Some(Opcode::new("ADD A, H", operand!((Register::A), (Register::H)), 4)),
        Some(Opcode::new("ADD A, L", operand!((Register::A), (Register::L)), 4)),
        Some(Opcode::new("ADD A, [HL]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("ADD A, A", operand!((Register::A), (Register::A)), 4)),
        Some(Opcode::new("ADC A, B", operand!((Register::A), (Register::B)), 4)),
        Some(Opcode::new("ADC A, C", operand!((Register::A), (Register::C)), 4)),
        Some(Opcode::new("ADC A, D", operand!((Register::A), (Register::D)), 4)),
        Some(Opcode::new("ADC A, E", operand!((Register::A), (Register::E)), 4)),
        Some(Opcode::new("ADC A, H", operand!((Register::A), (Register::H)), 4)),
        Some(Opcode::new("ADC A, L", operand!((Register::A), (Register::L)), 4)),
        Some(Opcode::new("ADC A, [HL]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("ADC A, A", operand!((Register::A), (Register::A)), 4)),
        Some(Opcode::new("SUB B", operand!((Register::B)), 4)),
        Some(Opcode::new("SUB C", operand!((Register::C)), 4)),
        Some(Opcode::new("SUB D", operand!((Register::D)), 4)),
        Some(Opcode::new("SUB E", operand!((Register::E)), 4)),
        Some(Opcode::new("SUB H", operand!((Register::H)), 4)),
        Some(Opcode::new("SUB L", operand!((Register::L)), 4)),
        Some(Opcode::new("SUB [HL]", operand!([(Register::H, Register::L)]), 8)),
        Some(Opcode::new("SUB A", operand!((Register::A)), 4)),
        Some(Opcode::new("SBC A, B", operand!((Register::A), (Register::B)), 4)),
        Some(Opcode::new("SBC A, C", operand!((Register::A), (Register::C)), 4)),
        Some(Opcode::new("SBC A, D", operand!((Register::A), (Register::D)), 4)),
        Some(Opcode::new("SBC A, E", operand!((Register::A), (Register::E)), 4)),
        Some(Opcode::new("SBC A, H", operand!((Register::A), (Register::H)), 4)),
        Some(Opcode::new("SBC A, L", operand!((Register::A), (Register::L)), 4)),
        Some(Opcode::new("SBC A, [HL]", operand!((Register::A), [(Register::H, Register::L)]), 8)),
        Some(Opcode::new("SBC A, A", operand!((Register::A), (Register::A)), 4)),
        Some(Opcode::new("AND B", operand!((Register::B)), 4)),
        Some(Opcode::new("AND C", operand!((Register::C)), 4)),
        Some(Opcode::new("AND D", operand!((Register::D)), 4)),
        Some(Opcode::new("AND E", operand!((Register::E)), 4)),
        Some(Opcode::new("AND H", operand!((Register::H)), 4)),
        Some(Opcode::new("AND L", operand!((Register::L)), 4)),
        Some(Opcode::new("AND [HL]", operand!([(Register::H, Register::L)]), 8)),
        Some(Opcode::new("AND A", operand!((Register::A)), 4)),
        Some(Opcode::new("XOR B", operand!((Register::B)), 4)),
        Some(Opcode::new("XOR C", operand!((Register::C)), 4)),
        Some(Opcode::new("XOR D", operand!((Register::D)), 4)),
        Some(Opcode::new("XOR E", operand!((Register::E)), 4)),
        Some(Opcode::new("XOR H", operand!((Register::H)), 4)),
        Some(Opcode::new("XOR L", operand!((Register::L)), 4)),
        Some(Opcode::new("XOR [HL]", operand!([(Register::H, Register::L)]), 8)),
        Some(Opcode::new("XOR A", operand!((Register::A)), 4)),
        Some(Opcode::new("OR B", operand!((Register::B)), 4)),
        Some(Opcode::new("OR C", operand!((Register::C)), 4)),
        Some(Opcode::new("OR D", operand!((Register::D)), 4)),
        Some(Opcode::new("OR E", operand!((Register::E)), 4)),
        Some(Opcode::new("OR H", operand!((Register::H)), 4)),
        Some(Opcode::new("OR L", operand!((Register::L)), 4)),
        Some(Opcode::new("OR [HL]", operand!([(Register::H, Register::L)]), 8)),
        Some(Opcode::new("OR A", operand!((Register::A)), 4)),
        Some(Opcode::new("CP B", operand!((Register::B)), 4)),
        Some(Opcode::new("CP C", operand!((Register::C)), 4)),
        Some(Opcode::new("CP D", operand!((Register::D)), 4)),
        Some(Opcode::new("CP E", operand!((Register::E)), 4)),
        Some(Opcode::new("CP H", operand!((Register::H)), 4)),
        Some(Opcode::new("CP L", operand!((Register::L)), 4)),
        Some(Opcode::new("CP [HL]", operand!([(Register::H, Register::L)]), 8)),
        Some(Opcode::new("CP A", operand!((Register::A)), 4)),
        Some(Opcode::new("RET NZ", operand!(), 20)),
        Some(Opcode::new("POP BC", operand!((Register::B, Register::C)), 12)),
        Some(Opcode::new("JP NZ, a16", operand!([U16]), 16)),
        Some(Opcode::new("JP a16", operand!([U16]), 16)),
        Some(Opcode::new("CALL NZ, a16", operand!([U16]), 24)),
        Some(Opcode::new("PUSH BC", operand!((Register::B, Register::C)), 16)),
        Some(Opcode::new("ADD A, d8", operand!((Register::A), U8), 8)),
        Some(Opcode::new("RST 00H", operand!(), 16)),
        Some(Opcode::new("RET Z", operand!(), 20)),
        Some(Opcode::new("RET", operand!(), 16)),
        Some(Opcode::new("JP Z, a16", operand!([U16]), 16)),
        Some(Opcode::new("PREFIX CB", operand!(), 4)),
        Some(Opcode::new("CALL Z, a16", operand!([U16]), 24)),
        Some(Opcode::new("CALL a16", operand!([U16]), 24)),
        Some(Opcode::new("ADC A, d8", operand!((Register::A), U8), 8)),
        Some(Opcode::new("RST 08H", operand!(), 16)),
        Some(Opcode::new("RET NC", operand!(), 20)),
        Some(Opcode::new("POP DE", operand!((Register::D, Register::E)), 12)),
        Some(Opcode::new("JP NC, a16", operand!([U16]), 16)),
        None, 
        Some(Opcode::new("CALL NC, a16", operand!([U16]), 24)),
        Some(Opcode::new("PUSH DE", operand!((Register::D, Register::E)), 16)),
        Some(Opcode::new("SUB d8", operand!(U8), 8)),
        Some(Opcode::new("RST 10H", operand!(), 16)),
        Some(Opcode::new("RET C", operand!(), 20)),
        Some(Opcode::new("RETI", operand!(), 16)),
        Some(Opcode::new("JP C, a16", operand!([U16]), 16)),
        None, 
        Some(Opcode::new("CALL C, a16", operand!([U16]), 24)),
        None, 
        Some(Opcode::new("SBC A, d8", operand!((Register::A), U8), 8)),
        Some(Opcode::new("RST 18H", operand!(), 16)),
        Some(Opcode::new("LDH [a8], A", operand!([U8], (Register::A)), 12)),
        Some(Opcode::new("POP HL", operand!((Register::H, Register::L)), 12)),
        Some(Opcode::new("LD [C], A", operand!([(Register::C)], (Register::A)), 8)),
        None, 
        None, 
        Some(Opcode::new("PUSH HL", operand!((Register::H, Register::L)), 16)),
        Some(Opcode::new("AND d8", operand!(U8), 8)),
        Some(Opcode::new("RST 20H", operand!(), 16)),
        Some(Opcode::new("ADD SP, r8", operand!((Register::SP), I8), 16)),
        Some(Opcode::new("JP HL", operand!((Register::H, Register::L)), 4)),
        Some(Opcode::new("LD [a16], A", operand!([U16], (Register::A)), 16)),
        None, 
        None, 
        None, 
        Some(Opcode::new("XOR d8", operand!(U8), 8)),
        Some(Opcode::new("RST 28H", operand!(), 16)),
        Some(Opcode::new("LDH A, [a8]", operand!((Register::A), [U8]), 12)),
        Some(Opcode::new("POP AF", operand!((Register::A, Register::F)), 12)),
        Some(Opcode::new("LD A, [C]", operand!((Register::A), [(Register::C)]), 8)),
        Some(Opcode::new("DI", operand!(), 4)),
        None,
        Some(Opcode::new("PUSH AF", operand!((Register::A, Register::F)), 16)),
        Some(Opcode::new("OR d8", operand!(U8), 8)),
        Some(Opcode::new("RST 30H", operand!(), 16)),
        Some(Opcode::new("LD HL, SP+r8", operand!((Register::H, Register::L), I8), 12)),
        Some(Opcode::new("LD SP, HL", operand!((Register::SP), (Register::H, Register::L)), 8)),
        Some(Opcode::new("LD A, [a16]", operand!((Register::A), [U16]), 16)),
        Some(Opcode::new("EI", operand!(), 4)),
        None,
        None,
        Some(Opcode::new("CP d8", operand!(U8), 8)),
        Some(Opcode::new("RST 38H", operand!(), 16)),
    ]
}

pub fn build_prefixed_map() -> [Opcode; 256] {
    [
        Opcode::new("RLC B", operand!((Register::B)), 8),
        Opcode::new("RLC C", operand!((Register::C)), 8),
        Opcode::new("RLC D", operand!((Register::D)), 8),
        Opcode::new("RLC E", operand!((Register::E)), 8),
        Opcode::new("RLC H", operand!((Register::H)), 8),
        Opcode::new("RLC L", operand!((Register::L)), 8),
        Opcode::new("RLC [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RLC A", operand!((Register::A)), 8),
        Opcode::new("RRC B", operand!((Register::B)), 8),
        Opcode::new("RRC C", operand!((Register::C)), 8),
        Opcode::new("RRC D", operand!((Register::D)), 8),
        Opcode::new("RRC E", operand!((Register::E)), 8),
        Opcode::new("RRC H", operand!((Register::H)), 8),
        Opcode::new("RRC L", operand!((Register::L)), 8),
        Opcode::new("RRC [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RRC A", operand!((Register::A)), 8),
        Opcode::new("RL B", operand!((Register::B)), 8),
        Opcode::new("RL C", operand!((Register::C)), 8),
        Opcode::new("RL D", operand!((Register::D)), 8),
        Opcode::new("RL E", operand!((Register::E)), 8),
        Opcode::new("RL H", operand!((Register::H)), 8),
        Opcode::new("RL L", operand!((Register::L)), 8),
        Opcode::new("RL [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RL A", operand!((Register::A)), 8),
        Opcode::new("RR B", operand!((Register::B)), 8),
        Opcode::new("RR C", operand!((Register::C)), 8),
        Opcode::new("RR D", operand!((Register::D)), 8),
        Opcode::new("RR E", operand!((Register::E)), 8),
        Opcode::new("RR H", operand!((Register::H)), 8),
        Opcode::new("RR L", operand!((Register::L)), 8),
        Opcode::new("RR [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RR A", operand!((Register::A)), 8),
        Opcode::new("SLA B", operand!((Register::B)), 8),
        Opcode::new("SLA C", operand!((Register::C)), 8),
        Opcode::new("SLA D", operand!((Register::D)), 8),
        Opcode::new("SLA E", operand!((Register::E)), 8),
        Opcode::new("SLA H", operand!((Register::H)), 8),
        Opcode::new("SLA L", operand!((Register::L)), 8),
        Opcode::new("SLA [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SLA A", operand!((Register::A)), 8),
        Opcode::new("SRA B", operand!((Register::B)), 8),
        Opcode::new("SRA C", operand!((Register::C)), 8),
        Opcode::new("SRA D", operand!((Register::D)), 8),
        Opcode::new("SRA E", operand!((Register::E)), 8),
        Opcode::new("SRA H", operand!((Register::H)), 8),
        Opcode::new("SRA L", operand!((Register::L)), 8),
        Opcode::new("SRA [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SRA A", operand!((Register::A)), 8),
        Opcode::new("SWAP B", operand!((Register::B)), 8),
        Opcode::new("SWAP C", operand!((Register::C)), 8),
        Opcode::new("SWAP D", operand!((Register::D)), 8),
        Opcode::new("SWAP E", operand!((Register::E)), 8),
        Opcode::new("SWAP H", operand!((Register::H)), 8),
        Opcode::new("SWAP L", operand!((Register::L)), 8),
        Opcode::new("SWAP [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SWAP A", operand!((Register::A)), 8),
        Opcode::new("SRL B", operand!((Register::B)), 8),
        Opcode::new("SRL C", operand!((Register::C)), 8),
        Opcode::new("SRL D", operand!((Register::D)), 8),
        Opcode::new("SRL E", operand!((Register::E)), 8),
        Opcode::new("SRL H", operand!((Register::H)), 8),
        Opcode::new("SRL L", operand!((Register::L)), 8),
        Opcode::new("SRL [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SRL A", operand!((Register::A)), 8),
        Opcode::new("BIT 0, B", operand!((Register::B)), 8),
        Opcode::new("BIT 0, C", operand!((Register::C)), 8),
        Opcode::new("BIT 0, D", operand!((Register::D)), 8),
        Opcode::new("BIT 0, E", operand!((Register::E)), 8),
        Opcode::new("BIT 0, H", operand!((Register::H)), 8),
        Opcode::new("BIT 0, L", operand!((Register::L)), 8),
        Opcode::new("BIT 0, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 0, A", operand!((Register::A)), 8),
        Opcode::new("BIT 1, B", operand!((Register::B)), 8),
        Opcode::new("BIT 1, C", operand!((Register::C)), 8),
        Opcode::new("BIT 1, D", operand!((Register::D)), 8),
        Opcode::new("BIT 1, E", operand!((Register::E)), 8),
        Opcode::new("BIT 1, H", operand!((Register::H)), 8),
        Opcode::new("BIT 1, L", operand!((Register::L)), 8),
        Opcode::new("BIT 1, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 1, A", operand!((Register::A)), 8),
        Opcode::new("BIT 2, B", operand!((Register::B)), 8),
        Opcode::new("BIT 2, C", operand!((Register::C)), 8),
        Opcode::new("BIT 2, D", operand!((Register::D)), 8),
        Opcode::new("BIT 2, E", operand!((Register::E)), 8),
        Opcode::new("BIT 2, H", operand!((Register::H)), 8),
        Opcode::new("BIT 2, L", operand!((Register::L)), 8),
        Opcode::new("BIT 2, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 2, A", operand!((Register::A)), 8),
        Opcode::new("BIT 3, B", operand!((Register::B)), 8),
        Opcode::new("BIT 3, C", operand!((Register::C)), 8),
        Opcode::new("BIT 3, D", operand!((Register::D)), 8),
        Opcode::new("BIT 3, E", operand!((Register::E)), 8),
        Opcode::new("BIT 3, H", operand!((Register::H)), 8),
        Opcode::new("BIT 3, L", operand!((Register::L)), 8),
        Opcode::new("BIT 3, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 3, A", operand!((Register::A)), 8),
        Opcode::new("BIT 4, B", operand!((Register::B)), 8),
        Opcode::new("BIT 4, C", operand!((Register::C)), 8),
        Opcode::new("BIT 4, D", operand!((Register::D)), 8),
        Opcode::new("BIT 4, E", operand!((Register::E)), 8),
        Opcode::new("BIT 4, H", operand!((Register::H)), 8),
        Opcode::new("BIT 4, L", operand!((Register::L)), 8),
        Opcode::new("BIT 4, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 4, A", operand!((Register::A)), 8),
        Opcode::new("BIT 5, B", operand!((Register::B)), 8),
        Opcode::new("BIT 5, C", operand!((Register::C)), 8),
        Opcode::new("BIT 5, D", operand!((Register::D)), 8),
        Opcode::new("BIT 5, E", operand!((Register::E)), 8),
        Opcode::new("BIT 5, H", operand!((Register::H)), 8),
        Opcode::new("BIT 5, L", operand!((Register::L)), 8),
        Opcode::new("BIT 5, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 5, A", operand!((Register::A)), 8),
        Opcode::new("BIT 6, B", operand!((Register::B)), 8),
        Opcode::new("BIT 6, C", operand!((Register::C)), 8),
        Opcode::new("BIT 6, D", operand!((Register::D)), 8),
        Opcode::new("BIT 6, E", operand!((Register::E)), 8),
        Opcode::new("BIT 6, H", operand!((Register::H)), 8),
        Opcode::new("BIT 6, L", operand!((Register::L)), 8),
        Opcode::new("BIT 6, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 6, A", operand!((Register::A)), 8),
        Opcode::new("BIT 7, B", operand!((Register::B)), 8),
        Opcode::new("BIT 7, C", operand!((Register::C)), 8),
        Opcode::new("BIT 7, D", operand!((Register::D)), 8),
        Opcode::new("BIT 7, E", operand!((Register::E)), 8),
        Opcode::new("BIT 7, H", operand!((Register::H)), 8),
        Opcode::new("BIT 7, L", operand!((Register::L)), 8),
        Opcode::new("BIT 7, [HL]", operand!([(Register::H, Register::L)]), 12),
        Opcode::new("BIT 7, A", operand!((Register::A)), 8),
        Opcode::new("RES 0, B", operand!((Register::B)), 8),
        Opcode::new("RES 0, C", operand!((Register::C)), 8),
        Opcode::new("RES 0, D", operand!((Register::D)), 8),
        Opcode::new("RES 0, E", operand!((Register::E)), 8),
        Opcode::new("RES 0, H", operand!((Register::H)), 8),
        Opcode::new("RES 0, L", operand!((Register::L)), 8),
        Opcode::new("RES 0, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 0, A", operand!((Register::A)), 8),
        Opcode::new("RES 1, B", operand!((Register::B)), 8),
        Opcode::new("RES 1, C", operand!((Register::C)), 8),
        Opcode::new("RES 1, D", operand!((Register::D)), 8),
        Opcode::new("RES 1, E", operand!((Register::E)), 8),
        Opcode::new("RES 1, H", operand!((Register::H)), 8),
        Opcode::new("RES 1, L", operand!((Register::L)), 8),
        Opcode::new("RES 1, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 1, A", operand!((Register::A)), 8),
        Opcode::new("RES 2, B", operand!((Register::B)), 8),
        Opcode::new("RES 2, C", operand!((Register::C)), 8),
        Opcode::new("RES 2, D", operand!((Register::D)), 8),
        Opcode::new("RES 2, E", operand!((Register::E)), 8),
        Opcode::new("RES 2, H", operand!((Register::H)), 8),
        Opcode::new("RES 2, L", operand!((Register::L)), 8),
        Opcode::new("RES 2, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 2, A", operand!((Register::A)), 8),
        Opcode::new("RES 3, B", operand!((Register::B)), 8),
        Opcode::new("RES 3, C", operand!((Register::C)), 8),
        Opcode::new("RES 3, D", operand!((Register::D)), 8),
        Opcode::new("RES 3, E", operand!((Register::E)), 8),
        Opcode::new("RES 3, H", operand!((Register::H)), 8),
        Opcode::new("RES 3, L", operand!((Register::L)), 8),
        Opcode::new("RES 3, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 3, A", operand!((Register::A)), 8),
        Opcode::new("RES 4, B", operand!((Register::B)), 8),
        Opcode::new("RES 4, C", operand!((Register::C)), 8),
        Opcode::new("RES 4, D", operand!((Register::D)), 8),
        Opcode::new("RES 4, E", operand!((Register::E)), 8),
        Opcode::new("RES 4, H", operand!((Register::H)), 8),
        Opcode::new("RES 4, L", operand!((Register::L)), 8),
        Opcode::new("RES 4, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 4, A", operand!((Register::A)), 8),
        Opcode::new("RES 5, B", operand!((Register::B)), 8),
        Opcode::new("RES 5, C", operand!((Register::C)), 8),
        Opcode::new("RES 5, D", operand!((Register::D)), 8),
        Opcode::new("RES 5, E", operand!((Register::E)), 8),
        Opcode::new("RES 5, H", operand!((Register::H)), 8),
        Opcode::new("RES 5, L", operand!((Register::L)), 8),
        Opcode::new("RES 5, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 5, A", operand!((Register::A)), 8),
        Opcode::new("RES 6, B", operand!((Register::B)), 8),
        Opcode::new("RES 6, C", operand!((Register::C)), 8),
        Opcode::new("RES 6, D", operand!((Register::D)), 8),
        Opcode::new("RES 6, E", operand!((Register::E)), 8),
        Opcode::new("RES 6, H", operand!((Register::H)), 8),
        Opcode::new("RES 6, L", operand!((Register::L)), 8),
        Opcode::new("RES 6, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 6, A", operand!((Register::A)), 8),
        Opcode::new("RES 7, B", operand!((Register::B)), 8),
        Opcode::new("RES 7, C", operand!((Register::C)), 8),
        Opcode::new("RES 7, D", operand!((Register::D)), 8),
        Opcode::new("RES 7, E", operand!((Register::E)), 8),
        Opcode::new("RES 7, H", operand!((Register::H)), 8),
        Opcode::new("RES 7, L", operand!((Register::L)), 8),
        Opcode::new("RES 7, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("RES 7, A", operand!((Register::A)), 8),
        Opcode::new("SET 0, B", operand!((Register::B)), 8),
        Opcode::new("SET 0, C", operand!((Register::C)), 8),
        Opcode::new("SET 0, D", operand!((Register::D)), 8),
        Opcode::new("SET 0, E", operand!((Register::E)), 8),
        Opcode::new("SET 0, H", operand!((Register::H)), 8),
        Opcode::new("SET 0, L", operand!((Register::L)), 8),
        Opcode::new("SET 0, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 0, A", operand!((Register::A)), 8),
        Opcode::new("SET 1, B", operand!((Register::B)), 8),
        Opcode::new("SET 1, C", operand!((Register::C)), 8),
        Opcode::new("SET 1, D", operand!((Register::D)), 8),
        Opcode::new("SET 1, E", operand!((Register::E)), 8),
        Opcode::new("SET 1, H", operand!((Register::H)), 8),
        Opcode::new("SET 1, L", operand!((Register::L)), 8),
        Opcode::new("SET 1, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 1, A", operand!((Register::A)), 8),
        Opcode::new("SET 2, B", operand!((Register::B)), 8),
        Opcode::new("SET 2, C", operand!((Register::C)), 8),
        Opcode::new("SET 2, D", operand!((Register::D)), 8),
        Opcode::new("SET 2, E", operand!((Register::E)), 8),
        Opcode::new("SET 2, H", operand!((Register::H)), 8),
        Opcode::new("SET 2, L", operand!((Register::L)), 8),
        Opcode::new("SET 2, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 2, A", operand!((Register::A)), 8),
        Opcode::new("SET 3, B", operand!((Register::B)), 8),
        Opcode::new("SET 3, C", operand!((Register::C)), 8),
        Opcode::new("SET 3, D", operand!((Register::D)), 8),
        Opcode::new("SET 3, E", operand!((Register::E)), 8),
        Opcode::new("SET 3, H", operand!((Register::H)), 8),
        Opcode::new("SET 3, L", operand!((Register::L)), 8),
        Opcode::new("SET 3, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 3, A", operand!((Register::A)), 8),
        Opcode::new("SET 4, B", operand!((Register::B)), 8),
        Opcode::new("SET 4, C", operand!((Register::C)), 8),
        Opcode::new("SET 4, D", operand!((Register::D)), 8),
        Opcode::new("SET 4, E", operand!((Register::E)), 8),
        Opcode::new("SET 4, H", operand!((Register::H)), 8),
        Opcode::new("SET 4, L", operand!((Register::L)), 8),
        Opcode::new("SET 4, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 4, A", operand!((Register::A)), 8),
        Opcode::new("SET 5, B", operand!((Register::B)), 8),
        Opcode::new("SET 5, C", operand!((Register::C)), 8),
        Opcode::new("SET 5, D", operand!((Register::D)), 8),
        Opcode::new("SET 5, E", operand!((Register::E)), 8),
        Opcode::new("SET 5, H", operand!((Register::H)), 8),
        Opcode::new("SET 5, L", operand!((Register::L)), 8),
        Opcode::new("SET 5, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 5, A", operand!((Register::A)), 8),
        Opcode::new("SET 6, B", operand!((Register::B)), 8),
        Opcode::new("SET 6, C", operand!((Register::C)), 8),
        Opcode::new("SET 6, D", operand!((Register::D)), 8),
        Opcode::new("SET 6, E", operand!((Register::E)), 8),
        Opcode::new("SET 6, H", operand!((Register::H)), 8),
        Opcode::new("SET 6, L", operand!((Register::L)), 8),
        Opcode::new("SET 6, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 6, A", operand!((Register::A)), 8),
        Opcode::new("SET 7, B", operand!((Register::B)), 8),
        Opcode::new("SET 7, C", operand!((Register::C)), 8),
        Opcode::new("SET 7, D", operand!((Register::D)), 8),
        Opcode::new("SET 7, E", operand!((Register::E)), 8),
        Opcode::new("SET 7, H", operand!((Register::H)), 8),
        Opcode::new("SET 7, L", operand!((Register::L)), 8),
        Opcode::new("SET 7, [HL]", operand!([(Register::H, Register::L)]), 16),
        Opcode::new("SET 7, A", operand!((Register::A)), 8),
    ]
}