use core::panic;

use crate::core::{
    opcode::*,
    system::System
};

pub fn nop(_opcode: &Opcode, _system: &mut System){}

pub fn ld(opcode: &Opcode, system: &mut System){
    let (op1, op2) =
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] A ld instruction should have an binary operand set")
        };

    let raw = opcode.raw;
    
    match raw{
        /* LD HL, SP + e8 */
        0xF8 => {
            let result = op2.get_value(system) + system.sp as usize;
            
            op1.update(
                result, 
                system
            );
            
            system.set_zflag(false);
            system.set_nflag(false);
            system.set_hflag((result & (1 << 3)) != 0);
            system.set_cflag((result & (1 << 7)) != 0);
        }
        /* LD A, [HL-] | LD A, [HL+] */
        0x3A | 0x2A => {
            op1.update(
                op2.get_value(system), 
                system
            );

            op2.update(
                if raw == 0x3A{op2.get_value(system) - 1} 
                else {op2.get_value(system) + 1},
                system
            );
        }
        /* LD [HL-], A | LD [HL+], A */
        0x32 | 0x22=> {
            op1.update(
                op2.get_value(system), 
                system
            );

            op1.update(
                if raw == 0x3A{op1.get_value(system) - 1} 
                else {op1.get_value(system) + 1},
                system
            );
        }
        _ => {
            op1.update(
                op2.get_value(system), 
                system
            );
        }
    }
    
}

pub fn ldh(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        if let Some(address) = op2.as_any().downcast_ref::<Address>() {
            op1.update(
                address.get_value_with_offset(
                    0xFF, 
                    system
                ), 
                system
            );
        }
        else if let Some(address) = op1.as_any().downcast_ref::<Address>() {
            address.update_with_offset(
                op2.get_value(system),
                 0xFF, 
                 system
            );
        }
        else{
            panic!("[ERROR] Invalid LDH opcode")
        }
    }
    else {
        panic!("[ERROR] LD instructions should have two operands");
    }
}

pub fn push(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] A push opcode should have an unary operand set");
        };
    let bytes = (operand.get_value(system) as u16).to_be_bytes();
    system.sp -= 1;
    system.wram[system.sp as usize] = bytes[1];
    system.sp -= 1;
    system.wram[system.sp as usize] = bytes[0];

}

pub fn pop(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] A push opcode should have an unary operand set");
        };
    
    let sp_hbyte = system.wram[system.sp as usize];
    system.sp += 1;
    let sp_lbyte = system.wram[system.sp as usize];
    system.sp += 1;

    operand.update(
        u16::from_be_bytes([sp_hbyte, sp_lbyte]) as usize, 
        system
    );
}

/* NOTE: This function is used with ADC and ADD */
pub fn add(opcode: &Opcode, system: &mut System){
    if let OperandSet::Unary(ref op) = opcode.operand {
        let result = 
            if 
                opcode.raw == 0x88 ||
                opcode.raw == 0x8E ||
                opcode.raw == 0xCE 
            {
                op.get_value(system) + system.get_cflag() as usize + system.a as usize
            }
            else {
                op.get_value(system) +  system.a as usize
            };
        
        op.update(result, system);

        system.set_zflag(result == 0);
        system.set_nflag(false);
        system.set_hflag((result & (1 << 3)) != 0);
        system.set_cflag((result & (1 << 7)) != 0);
    }
    else if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
        let result = op1.get_value(system) + op2.get_value(system);
        
        op1.update(result, system);

        system.set_nflag(false);
        if opcode.raw == 0xE8{
            system.set_zflag(false);    
            system.set_hflag((result & (1 << 11)) != 0);
            system.set_cflag((result & (1 << 15)) != 0);
        }
        else {
            system.set_hflag((result & (1 << 3)) != 0);
            system.set_cflag((result & (1 << 7)) != 0);
        }
    }
    else {
        panic!("[ERROR] An add/adc opcode should have an unary/binary operand set");
    }
}

/* NOTE: This function is used with SBC and SUB */
pub fn sub(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] An sub/sbc opcode should have an unary operand set");
        };
    
    let result = 
        if 
            opcode.raw == 0x98 ||
            opcode.raw == 0x9E ||
            opcode.raw == 0xDE 
        {
            system.a as usize - operand.get_value(system) - system.get_cflag() as usize 
        }
        else {
            system.a as usize - operand.get_value(system) 
        };

    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(true);
    system.set_hflag((result & (1 << 3)) != 0);
    system.set_cflag((result & (1 << 7)) != 0);

}

pub fn cp(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A cp opcode should have an unary operand set");
        };
    
    let result = operand.get_value(system) -  system.a as usize;
    
    system.set_zflag(result == 0);
    system.set_nflag(true);
    system.set_hflag((result & (1 << 3)) != 0);
    system.set_cflag((result & (1 << 7)) != 0);
}

pub fn inc(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] An inc opcode should have an unary operand set");
        };
    
    let result = operand.get_value(system) + 1;
    
    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    if is_16bit_operand(operand.as_ref()){
        system.set_hflag((result & (1 << 11)) != 0);
    }
    else {
        system.set_hflag((result & (1 << 3)) != 0);
    }
}

pub fn dec(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A dec opcode should have an unary operand set");
        };
    
    let result = operand.get_value(system) - 1;
    
    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(true);
    
    if is_16bit_operand(operand.as_ref()){
        system.set_hflag((result & (1 << 11)) != 0);
    }
    else {
        system.set_hflag((result & (1 << 3)) != 0);
    }
}

pub fn and(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A and opcode should have an unary operand set");
        };
    
    let result = system.a as usize & operand.get_value(system);
    
    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(true);
    system.set_cflag(false);
}

pub fn or(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A or opcode should have an unary operand set");
        };
    
    let result = system.a as usize | operand.get_value(system);
    
    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(false);
}

pub fn xor(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A xor opcode should have an unary operand set");
        };
    
    let result = system.a as usize ^ operand.get_value(system);
    
    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(false);
}

pub fn ccf(opcode: &Opcode, system: &mut System){
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(!system.get_cflag());
}

pub fn scf(opcode: &Opcode, system: &mut System){
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(true);
}

pub fn daa(opcode: &Opcode, system: &mut System){

}

pub fn cpl(opcode: &Opcode, system: &mut System){
    system.a = !system.a;
    system.set_nflag(true);
    system.set_hflag(true);
}