use crate::core::{
    opcode::*,
    system::System
};

pub fn nop(_opcode: &Opcode, _system: &mut System){}

pub fn ld(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        let raw = opcode.raw;
        match raw{
            /* LD HL, SP + e8 */
            0xF8 => {
                op1.update(
                    op2.get_value(system) + system.sp as usize, 
                    0,
                    system
                );
            }
            /* LD A, [HL-] | LD A, [HL+]*/
            0x3A | 0x2A => {
                op1.update(
                    op2.get_value(system), 
                    0,
                    system
                );

                op2.update(
                    if raw == 0x3A{op2.get_value(system) - 1} 
                    else {op2.get_value(system) + 1}, 
                    0,
                    system
                );
            }
            /* LD [HL-], A | LD [HL+], A*/
            0x32 | 0x22=> {
                op1.update(
                    op2.get_value(system), 
                    0,
                    system
                );

                op1.update(
                    if raw == 0x3A{op1.get_value(system) - 1} 
                    else {op1.get_value(system) + 1}, 
                    0,
                    system
                );
            }
            _ => {
                op1.update(
                    op2.get_value(system), 
                    0,
                    system
                );
            }
        }
    }
    else {
        panic!("[ERROR] LD instructions should have two operands");
    }
}

pub fn ldh(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        op1.update(
            op2.get_value(system),
            0xFF, 
            system
        );
    }
    else {
        panic!("[ERROR] LD instructions should have two operands");
    }
}

pub fn inc(opcode: &Opcode, system: &mut System){}

pub fn dec(opcode: &Opcode, system: &mut System){}

pub fn add(opcode: &Opcode, system: &mut System){}

pub fn sub(opcode: &Opcode, system: &mut System){}

pub fn rlca(opcode: &Opcode, system: &mut System){}

pub fn rrca(opcode: &Opcode, system: &mut System){}

pub fn rla(opcode: &Opcode, system: &mut System){}

pub fn rra(opcode: &Opcode, system: &mut System){}

pub fn daa(opcode: &Opcode, system: &mut System){}

pub fn cpl(opcode: &Opcode, system: &mut System){}

pub fn scf(opcode: &Opcode, system: &mut System){}

pub fn ccf(opcode: &Opcode, system: &mut System){}

pub fn jr(opcode: &Opcode, system: &mut System){}

pub fn stop(opcode: &Opcode, system: &mut System){}

pub fn halt(opcode: &Opcode, system: &mut System){}

pub fn adc(opcode: &Opcode, system: &mut System){}

pub fn sbc(opcode: &Opcode, system: &mut System){}

pub fn and(opcode: &Opcode, system: &mut System){}

pub fn xor(opcode: &Opcode, system: &mut System){}

pub fn or(opcode: &Opcode, system: &mut System){}

pub fn cp(opcode: &Opcode, system: &mut System){}

pub fn ret(opcode: &Opcode, system: &mut System){}

pub fn reti(opcode: &Opcode, system: &mut System){}

pub fn jp(opcode: &Opcode, system: &mut System){}

pub fn call(opcode: &Opcode, system: &mut System){}

pub fn rst(opcode: &Opcode, system: &mut System){}

pub fn pop(opcode: &Opcode, system: &mut System){}

pub fn push(opcode: &Opcode, system: &mut System){}

pub fn di(opcode: &Opcode, system: &mut System){}

pub fn ei(opcode: &Opcode, system: &mut System){}

pub fn rlc(opcode: &Opcode, system: &mut System){}

pub fn rrc(opcode: &Opcode, system: &mut System){}

pub fn rl(opcode: &Opcode, system: &mut System){}

pub fn rr(opcode: &Opcode, system: &mut System){}

pub fn sla(opcode: &Opcode, system: &mut System){}

pub fn sra(opcode: &Opcode, system: &mut System){}

pub fn swap(opcode: &Opcode, system: &mut System){}

pub fn srl(opcode: &Opcode, system: &mut System){}

pub fn bit(opcode: &Opcode, system: &mut System){}

pub fn res(opcode: &Opcode, system: &mut System){}

pub fn set(opcode: &Opcode, system: &mut System){}