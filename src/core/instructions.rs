use core::panic;

use crate::core::{
    opcode::*,
    system::System
};
  

/* TODO: HALT
 * TODO: STOP
 * TODO: EI 
 * TODO: DAA
 * TODO: RETI
 */

const HC_8B_MASK: usize = 0xF;
const HC_16B_MASK: usize = 0xFF;
const C_8B_MASK: usize = 0xFF;
const C_16B_MAKS: usize = 0xFFFF;

pub fn nop(_opcode: &Opcode, _system: &mut System){}

pub fn ld(opcode: &Opcode, system: &mut System){
    let (op1, op2) =
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] A ld instruction should have an binary operand set");
        };

    let raw = opcode.raw;
    
    match raw{
        /* LD HL, SP + e8 */
        0xF8 => {
            let result = op2.get_value(system) + system.sp as usize;
            let hc_result = (op2.get_value(system) & HC_8B_MASK) + (system.sp as usize & HC_8B_MASK); 
            op1.update(
                result, 
                system
            );
            
            system.set_zflag(false);
            system.set_nflag(false);
            system.set_hflag(hc_result > HC_8B_MASK);
            system.set_cflag(result > C_8B_MASK);
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
    if 
        let OperandSet::Binary(ref op1, ref op2) = opcode.operand &&
        (!is_16bit_operand(op1.as_ref()) && !is_16bit_operand(op2.as_ref())) 
    {
        let mut result = op1.get_value(system) + op2.get_value(system);
        let mut hc_result = (op1.get_value(system) & HC_8B_MASK) + (op2.get_value(system) & HC_8B_MASK); 
        
        /* ADC instructions */
        if 
            opcode.raw == 0x88 ||
            opcode.raw == 0x8E ||
            opcode.raw == 0xCE 
        {
            result += system.get_cflag() as usize;
            hc_result += system.get_cflag() as usize; 
        }

        op1.update(result, system);

        system.set_zflag(result == 0);
        system.set_nflag(false);
        system.set_hflag(hc_result > HC_8B_MASK);
        system.set_cflag(result > C_8B_MASK);
    }
    else if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
        let result = op1.get_value(system) + op2.get_value(system);
        
        let (c_mask, hc_mask) = 
            if opcode.raw == 0xE8 {
                system.set_zflag(false);
                (C_16B_MAKS, HC_16B_MASK)
            }
            else{
                (C_8B_MASK, HC_8B_MASK)
            };

        let hc_result= (op1.get_value(system) & hc_mask) + (op2.get_value(system) & hc_mask);

        op1.update(result, system);
        system.set_nflag(false);
        system.set_hflag(hc_result > hc_mask);
        system.set_cflag(result > c_mask);
        
    }
    else {
        panic!("[ERROR] An add/adc opcode should have an unary/binary operand set");
    }
}

/* NOTE: This function is used with SBC and SUB */
pub fn sub(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] An sub/sbc opcode should have an binary operand set");
        };
    
    let mut result = op1.get_value(system) - op2.get_value(system);
    let mut hc_result = (op1.get_value(system) & HC_8B_MASK) - (op2.get_value(system) & HC_8B_MASK);
    
    /* SBC instructions */
    if 
        opcode.raw == 0x98 ||
        opcode.raw == 0x9E ||
        opcode.raw == 0xDE 
    {
        result -= system.get_cflag() as usize; 
        hc_result -= system.get_cflag() as usize;
    }

    op1.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(true);
    system.set_hflag(hc_result > HC_8B_MASK);
    system.set_cflag(result > C_8B_MASK);

}

pub fn cp(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] An cp opcode should have an binary operand set");
        };
    
    let result = op1.get_value(system) - op2.get_value(system);
    let hc_result = (op1.get_value(system) & HC_8B_MASK) - (op2.get_value(system) & HC_8B_MASK);
    op1.update(result, system);
    
    system.set_zflag(result == 0);
    system.set_nflag(true);
    system.set_hflag(hc_result > HC_8B_MASK);
    system.set_cflag(result > C_8B_MASK);
}

pub fn inc(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] An inc opcode should have an unary operand set");
        };

    let (c_mask, hc_mask) = 
        if is_16bit_operand(operand.as_ref()){
            (C_16B_MAKS, HC_16B_MASK)
        }
        else {
            (C_8B_MASK, HC_8B_MASK)
        };
    
    let result = operand.get_value(system) + 1;
    let hc_result = (operand.get_value(system) & hc_mask) + 1;

    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(hc_result > hc_mask);
}

pub fn dec(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand {
            op
        }
        else {
            panic!("[ERROR] A dec opcode should have an unary operand set");
        };
    
    let (c_mask, hc_mask) = 
        if is_16bit_operand(operand.as_ref()){
            (C_16B_MAKS, HC_16B_MASK)
        }
        else {
            (C_8B_MASK, HC_8B_MASK)
        };
    
    let result = operand.get_value(system) - 1;
    let hc_result = (operand.get_value(system) & hc_mask) - 1;

    operand.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(hc_result > hc_mask);
}

pub fn and(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] An and opcode should have a binary operand set");
        };
    
    let result = op1.get_value(system) & op2.get_value(system);
    
    op1.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(true);
    system.set_cflag(false);
}

pub fn or(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] An or opcode should have a binary operand set");
        };
    
    let result = op1.get_value(system) | op2.get_value(system);
    
    op1.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(false);
}

pub fn xor(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] A xor opcode should have a binary operand set");
        };
    
    let result = op1.get_value(system) ^ op2.get_value(system);
    
    op1.update(result, system);

    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(false);
}

pub fn ccf(_opcode: &Opcode, system: &mut System){
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(!system.get_cflag());
}

pub fn scf(_opcode: &Opcode, system: &mut System){
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(true);
}

pub fn cpl(_opcode: &Opcode, system: &mut System){
    system.a = !system.a;
    system.set_nflag(true);
    system.set_hflag(true);
}

pub fn rlc(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] rlc instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rlc can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(system) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(system) << 1) | b7 as usize, 
        system
    );

    if opcode.raw == 0x07{
        system.set_zflag(false);
    }
    else {
        let result = operand.get_value(system);
        system.set_zflag(result == 0);
    }
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b7 != 0);
}

pub fn rl(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] rl instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rl can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(system) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(system) << 1) | system.get_cflag() as usize, 
        system
    );

    if opcode.raw == 0x17{
        system.set_zflag(false);
    }
    else {
        let result = operand.get_value(system);
        system.set_zflag(result == 0);
    }
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b7 != 0);
}

pub fn rrc(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] rrc instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rrc can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(system) & 1) as u8;
    operand.update(
        (operand.get_value(system) >> 1) | b0 as usize, 
        system
    );

    if opcode.raw == 0x0F{
        system.set_zflag(false);
    }
    else {
        let result = operand.get_value(system);
        system.set_zflag(result == 0);
    }
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b0 != 0);
}

pub fn rr(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] rr instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rr can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(system) & 1) as u8;
    operand.update(
        (operand.get_value(system) >> 1) | system.get_cflag() as usize, 
        system
    );

    if opcode.raw == 0x1F{
        system.set_zflag(false);
    }
    else {
        let result = operand.get_value(system);
        system.set_zflag(result == 0);
    }
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b0 != 0);
}

pub fn sla(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] sla instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] sla can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(system) & (1 << 7)) != 0) as u8;
    operand.update(
        operand.get_value(system) << 1, 
        system
    );

    let result = operand.get_value(system);
    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b7 != 0);
}

pub fn sra(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] sra instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] sra can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(system) & 1) as u8;
    let b7 = ((operand.get_value(system) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(system) >> 1) | (b7 << 7) as usize, 
        system
    );

    let result = operand.get_value(system);
    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b0 != 0);
}

pub fn swap(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] swap instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] swap can not be an 16 bit operand");
    }

    operand.update(
        (operand.get_value(system) & 0x0F) | (operand.get_value(system) & 0xF0), 
        system
    );

    let result = operand.get_value(system);
    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(false);
}

pub fn srl(opcode: &Opcode, system: &mut System){
    let operand = 
        if let OperandSet::Unary(ref op) = opcode.operand{
            op
        }
        else {
            panic!("[ERROR] srl instruction should have an unary operand set");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] srl can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(system) & 1) as u8;
    operand.update(
        operand.get_value(system) >> 1, 
        system
    );

    let result = operand.get_value(system);
    system.set_zflag(result == 0);
    system.set_nflag(false);
    system.set_hflag(false);
    system.set_cflag(b0 != 0);
}

pub fn bit(opcode: &Opcode, system: &mut System){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
            (op1.get_value(system), op2.get_value(system))
        }
        else {
            panic!("[ERROR] bit instruction should have a binary operand set");
        };

    system.set_zflag((op2 & (1 << op1)) != 0);
    system.set_nflag(false);
    system.set_hflag(true);
}

pub fn res(opcode: &Opcode, system: &mut System){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
            (op1.get_value(system), op2)
        }
        else {
            panic!("[ERROR] res instruction should have a binary operand set");
        };

    /*
        op2: 10110110
        op1: 4

        shift: ( 1 << 4 ) = 10000
        !shift: 11101111
        op2 & !shift: 10110110 & 11101111 = 10100110 
    */
    op2.update(
        op2.get_value(system) & !(1 << op1),
        system
    );
}

pub fn set(opcode: &Opcode, system: &mut System){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
            (op1.get_value(system), op2)
        }
        else {
            panic!("[ERROR] set instruction should have a binary operand set");
        };

    op2.update(
        op2.get_value(system) | (1 << op1),
        system
    );
}

pub fn jp(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        if op1.get_value(system) == 0 {return;}
        system.pc = op2.get_value(system) as u16;
    }
    else if let OperandSet::Unary(ref op) = opcode.operand {
        system.pc = op.get_value(system) as u16;
    }
    else {
        panic!("[ERROR] jp instruction should not have an unary operand set");
    }
}

pub fn jr(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        if op1.get_value(system) == 0 {return;}
        system.pc += op2.get_value(system) as u16;
    }
    else if let OperandSet::Unary(ref op) = opcode.operand {
        system.pc += op.get_value(system) as u16;
    }
    else {
        panic!("[ERROR] jr instruction should not have an unary operand set");
    }
}

pub fn call(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand {
        if op1.get_value(system) == 0 {return;}
        system.sp -= 1;
        system.wram[system.sp as usize] = (system.pc & 0x00FF) as u8;
        system.sp -= 1;
        system.wram[system.sp as usize] = ((system.pc & 0xFF00) >> 8) as u8;
        system.pc = op2.get_value(system) as u16;
    }
    /* NOTE: Can be used in CALL nn and RST $FF */
    else if let OperandSet::Unary(ref op) = opcode.operand {
        system.sp -= 1;
        system.wram[system.sp as usize] = (system.pc & 0x00FF) as u8;
        system.sp -= 1;
        system.wram[system.sp as usize] = ((system.pc & 0xFF00) >> 8) as u8;
        system.pc = op.get_value(system) as u16;
    }
    else {
        panic!("[ERROR] call/rst instruction should not have an unary operand set");
    }
}

pub fn ret(opcode: &Opcode, system: &mut System){
    let pc = 
        (system.wram[(system.sp + 1) as usize] as u16) << 8 |
        (system.wram[(system.sp) as usize] as u16);
    
    if let OperandSet::Nullary = opcode.operand{
        system.sp += 2;
        system.pc = pc;
    }
    else if let OperandSet::Unary(ref op) = opcode.operand {
        if op.get_value(system) == 0 {return;}
        system.sp += 2;
        system.pc = pc;
    }
    else {
        panic!("[ERROR] ret instruction should not have an binary operand set");
    }
}

