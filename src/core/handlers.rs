use core::panic;

use crate::core::{
    cpu::*, 
    instruction::*, 
    system::{
        Byte, 
        MEMORY_SIZE
    }
};
  

/* TODO: HALT
 * TODO: STOP
 * TODO: EI 
 * TODO: DAA
 * TODO: RETI
 * TODO: DI
 */

const HC_8B_MASK: usize = 0xF;
const HC_16B_MASK: usize = 0xFF;
const C_8B_MASK: usize = 0xFF;
const C_16B_MAKS: usize = 0xFFFF;

pub fn halt(_instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){}
pub fn stop(_instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){}
pub fn ei(_instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){}
pub fn daa(_instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){}
pub fn di(_instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){}


pub fn nop(instruction: &Instruction, _cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Nullary = instruction.operand { return; }
    panic!("[ERROR] nop should not have operands");
}

pub fn ld(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) =
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] ld should have two operands");
        };

    let opcode = instruction.opcode;
    
    match opcode{
        /* LD HL, SP + e8 */
        0xF8 => {
            let result = op2.get_value(cpu, ram) + cpu.sp as usize;
            let hc_result = (op2.get_value(cpu, ram) & HC_8B_MASK) + (cpu.sp as usize & HC_8B_MASK); 
            op1.update(
                result, 
                cpu,
                ram
            );
            
            cpu.set_zflag(false);
            cpu.set_nflag(false);
            cpu.set_hflag(hc_result > HC_8B_MASK);
            cpu.set_cflag(result > C_8B_MASK);
        }
        /* LD A, [HL-] | LD A, [HL+] */
        0x3A | 0x2A => {
            op1.update(
                op2.get_value(cpu, ram), 
                cpu,
                ram
            );

            op2.update(
                if opcode == 0x3A{op2.get_value(cpu, ram) - 1} 
                else {op2.get_value(cpu, ram) + 1},
                cpu,
                ram
            );
        }
        /* LD [HL-], A | LD [HL+], A */
        0x32 | 0x22=> {
            op1.update(
                op2.get_value(cpu, ram), 
                cpu,
                ram
            );

            op1.update(
                if opcode == 0x3A{op1.get_value(cpu, ram) - 1} 
                else {op1.get_value(cpu, ram) + 1},
                cpu,
                ram
            );
        }
        _ => {
            op1.update(
                op2.get_value(cpu, ram), 
                cpu,
                ram
            );
        }
    }
    
}

pub fn ldh(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
        if let Some(address) = op2.as_any().downcast_ref::<Address>() {
            op1.update(
                address.get_value_with_offset(
                    0xFF, 
                    cpu,
                    ram
                ), 
                cpu,
                ram
            );
        }
        else if let Some(address) = op1.as_any().downcast_ref::<Address>() {
            address.update_with_offset(
                op2.get_value(cpu, ram),
                 0xFF, 
                 cpu,
                 ram
            );
        }
        else{
            panic!("[ERROR] Invalid LDH instruction")
        }
    }
    else {
        panic!("[ERROR] ldh should have two operands");
    }
}

pub fn push(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] push should have one operand");
        };

    let bytes = (operand.get_value(cpu, ram) as u16).to_be_bytes();
    cpu.sp -= 1;
    ram[cpu.sp as usize] = bytes[1];
    cpu.sp -= 1;
    ram[cpu.sp as usize] = bytes[0];

}

pub fn pop(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] pop should have one operand");
        };
    
    let sp_hbyte = ram[cpu.sp as usize];
    cpu.sp += 1;
    let sp_lbyte = ram[cpu.sp as usize];
    cpu.sp += 1;

    operand.update(
        u16::from_be_bytes([sp_hbyte, sp_lbyte]) as usize, 
        cpu,
        ram
    );
}

/* NOTE: This function is used with ADC and ADD */
pub fn add(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    if 
        let OperandSet::Binary(ref op1, ref op2) = instruction.operand &&
        (!is_16bit_operand(op1.as_ref()) && !is_16bit_operand(op2.as_ref())) 
    {
        let mut result = op1.get_value(cpu, ram) + op2.get_value(cpu, ram);
        let mut hc_result = (op1.get_value(cpu, ram) & HC_8B_MASK) + (op2.get_value(cpu, ram) & HC_8B_MASK); 
        
        /* ADC instructions */
        if 
            instruction.opcode == 0x88 ||
            instruction.opcode == 0x8E ||
            instruction.opcode == 0xCE 
        {
            result += cpu.get_cflag() as usize;
            hc_result += cpu.get_cflag() as usize; 
        }

        op1.update(result, cpu, ram);

        cpu.set_zflag(result == 0);
        cpu.set_nflag(false);
        cpu.set_hflag(hc_result > HC_8B_MASK);
        cpu.set_cflag(result > C_8B_MASK);
    }
    else if let OperandSet::Binary(ref op1, ref op2) = instruction.operand{
        let result = op1.get_value(cpu, ram) + op2.get_value(cpu, ram);
        
        let (c_mask, hc_mask) = 
            if instruction.opcode == 0xE8 {
                cpu.set_zflag(false);
                (C_16B_MAKS, HC_16B_MASK)
            }
            else{
                (C_8B_MASK, HC_8B_MASK)
            };

        let hc_result= (op1.get_value(cpu, ram) & hc_mask) + (op2.get_value(cpu, ram) & hc_mask);

        op1.update(result, cpu, ram);
        cpu.set_nflag(false);
        cpu.set_hflag(hc_result > hc_mask);
        cpu.set_cflag(result > c_mask);
        
    }
    else {
        panic!("[ERROR] add/adc instruction should have one or two operands");
    }
}

/* NOTE: This function is used with SBC and SUB */
pub fn sub(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] sub/sbc should have two operands");
        };
    
    let mut result = op1.get_value(cpu, ram) - op2.get_value(cpu, ram);
    let mut hc_result = (op1.get_value(cpu, ram) & HC_8B_MASK) - (op2.get_value(cpu, ram) & HC_8B_MASK);
    
    /* SBC instructions */
    if 
        instruction.opcode == 0x98 ||
        instruction.opcode == 0x9E ||
        instruction.opcode == 0xDE 
    {
        result -= cpu.get_cflag() as usize; 
        hc_result -= cpu.get_cflag() as usize;
    }

    op1.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(true);
    cpu.set_hflag(hc_result > HC_8B_MASK);
    cpu.set_cflag(result > C_8B_MASK);

}

pub fn cp(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] cp should have two operands");
        };
    
    let result = op1.get_value(cpu, ram) - op2.get_value(cpu, ram);
    let hc_result = (op1.get_value(cpu, ram) & HC_8B_MASK) - (op2.get_value(cpu, ram) & HC_8B_MASK);
    op1.update(result, cpu, ram);
    
    cpu.set_zflag(result == 0);
    cpu.set_nflag(true);
    cpu.set_hflag(hc_result > HC_8B_MASK);
    cpu.set_cflag(result > C_8B_MASK);
}

pub fn inc(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand {
            op
        }
        else {
            panic!("[ERROR] inc should have one operand");
        };

    let hc_mask = 
        if is_16bit_operand(operand.as_ref()){
            HC_16B_MASK
        }
        else {
            HC_8B_MASK
        };
    
    let result = operand.get_value(cpu, ram) + 1;
    let hc_result = (operand.get_value(cpu, ram) & hc_mask) + 1;

    operand.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(hc_result > hc_mask);
}

pub fn dec(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand {
            op
        }
        else {
            panic!("[ERROR] dec should have one operand");
        };
    
    let hc_mask = 
        if is_16bit_operand(operand.as_ref()){
            HC_16B_MASK
        }
        else {
            HC_8B_MASK
        };
    
    let result = operand.get_value(cpu, ram) - 1;
    let hc_result = (operand.get_value(cpu, ram) & hc_mask) - 1;

    operand.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(hc_result > hc_mask);
}

pub fn and(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] and should have two operands");
        };
    
    let result = op1.get_value(cpu, ram) & op2.get_value(cpu, ram);
    
    op1.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(true);
    cpu.set_cflag(false);
}

pub fn or(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] or should have two operands");
        };
    
    let result = op1.get_value(cpu, ram) | op2.get_value(cpu, ram);
    
    op1.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn xor(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
            (op1, op2)
        }
        else {
            panic!("[ERROR] xor should have two operand");
        };
    
    let result = op1.get_value(cpu, ram) ^ op2.get_value(cpu, ram);
    
    op1.update(result, cpu, ram);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn ccf(instruction: &Instruction, cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Nullary = instruction.operand { 
        cpu.set_nflag(false);
        cpu.set_hflag(false);
        cpu.set_cflag(!cpu.get_cflag());
        return; 
    }
    panic!("[ERROR] ccf should not have operands");
}

pub fn scf(instruction: &Instruction, cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Nullary = instruction.operand { 
        cpu.set_nflag(false);
        cpu.set_hflag(false);
        cpu.set_cflag(true);
        return; 
    }
    panic!("[ERROR] scf should not have operands");
}

pub fn cpl(instruction: &Instruction, cpu: &mut Cpu, _ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Nullary = instruction.operand { 
        cpu.a = !cpu.a;
        cpu.set_nflag(true);
        cpu.set_hflag(true);
        return; 
    }
    panic!("[ERROR] cpl should not have operands");
}

pub fn rlc(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] rlc should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rlc can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(cpu, ram) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(cpu, ram) << 1) | b7 as usize, 
        cpu,
        ram
    );

    if instruction.opcode == 0x07{
        cpu.set_zflag(false);
    }
    else {
        let result = operand.get_value(cpu, ram);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn rl(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] rl should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rl can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(cpu, ram) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(cpu, ram) << 1) | cpu.get_cflag() as usize, 
        cpu,
        ram
    );

    if instruction.opcode == 0x17{
        cpu.set_zflag(false);
    }
    else {
        let result = operand.get_value(cpu, ram);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn rrc(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] rrc should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rrc can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(cpu, ram) & 1) as u8;
    operand.update(
        (operand.get_value(cpu, ram) >> 1) | b0 as usize, 
        cpu,
        ram
    );

    if instruction.opcode == 0x0F{
        cpu.set_zflag(false);
    }
    else {
        let result = operand.get_value(cpu, ram);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn rr(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] rr should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] rr can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(cpu, ram) & 1) as u8;
    operand.update(
        (operand.get_value(cpu, ram) >> 1) | cpu.get_cflag() as usize, 
        cpu,
        ram
    );

    if instruction.opcode == 0x1F{
        cpu.set_zflag(false);
    }
    else {
        let result = operand.get_value(cpu, ram);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn sla(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] sla should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] sla can not be an 16 bit operand");
    }

    let b7 = ((operand.get_value(cpu, ram) & (1 << 7)) != 0) as u8;
    operand.update(
        operand.get_value(cpu, ram) << 1, 
        cpu,
        ram
    );

    let result = operand.get_value(cpu, ram);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn sra(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] sra should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] sra can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(cpu, ram) & 1) as u8;
    let b7 = ((operand.get_value(cpu, ram) & (1 << 7)) != 0) as u8;
    operand.update(
        (operand.get_value(cpu, ram) >> 1) | (b7 << 7) as usize, 
        cpu,
        ram
    );

    let result = operand.get_value(cpu, ram);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn swap(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] swap should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] swap can not be an 16 bit operand");
    }

    operand.update(
        (operand.get_value(cpu, ram) & 0x0F) | (operand.get_value(cpu, ram) & 0xF0), 
        cpu,
        ram
    );

    let result = operand.get_value(cpu, ram);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn srl(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let operand = 
        if let OperandSet::Unary(ref op) = instruction.operand{
            op
        }
        else {
            panic!("[ERROR] srl should have one operand");
        };
    
    if is_16bit_operand(operand.as_ref()) {
        panic!("[ERROR] srl can not be an 16 bit operand");
    }

    let b0 = (operand.get_value(cpu, ram) & 1) as u8;
    operand.update(
        operand.get_value(cpu, ram) >> 1, 
        cpu,
        ram
    );

    let result = operand.get_value(cpu, ram);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn bit(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand{
            (op1.get_value(cpu, ram), op2.get_value(cpu, ram))
        }
        else {
            panic!("[ERROR] bit should have two operands");
        };

    cpu.set_zflag((op2 & (1 << op1)) != 0);
    cpu.set_nflag(false);
    cpu.set_hflag(true);
}

pub fn res(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand{
            (op1.get_value(cpu, ram), op2)
        }
        else {
            panic!("[ERROR] res should have two operands");
        };

    /*
        op2: 10110110
        op1: 4

        shift: ( 1 << 4 ) = 10000
        !shift: 11101111
        op2 & !shift: 10110110 & 11101111 = 10100110 
    */
    op2.update(
        op2.get_value(cpu, ram) & !(1 << op1),
        cpu,
        ram
    );
}

pub fn set(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operand{
            (op1.get_value(cpu, ram), op2)
        }
        else {
            panic!("[ERROR] set should have two operands");
        };

    op2.update(
        op2.get_value(cpu, ram) | (1 << op1),
        cpu,
        ram
    );
}

pub fn jp(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
        if op1.get_value(cpu, ram) == 0 {return;}
        cpu.pc = op2.get_value(cpu, ram) as u16;
    }
    else if let OperandSet::Unary(ref op) = instruction.operand {
        cpu.pc = op.get_value(cpu, ram) as u16;
    }
    else {
        panic!("[ERROR] jp should have one or nonne operands");
    }
}

pub fn jr(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
        if op1.get_value(cpu, ram) == 0 {return;}
        cpu.pc += op2.get_value(cpu, ram) as u16;
    }
    else if let OperandSet::Unary(ref op) = instruction.operand {
        cpu.pc += op.get_value(cpu, ram) as u16;
    }
    else {
        panic!("[ERROR] jr should have one or nonne operands");
    }
}

pub fn call(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operand {
        if op1.get_value(cpu, ram) == 0 {return;}
        cpu.sp -= 1;
        ram[cpu.sp as usize] = (cpu.pc & 0x00FF) as u8;
        cpu.sp -= 1;
        ram[cpu.sp as usize] = ((cpu.pc & 0xFF00) >> 8) as u8;
        cpu.pc = op2.get_value(cpu, ram) as u16;
    }
    /* NOTE: Can be used in CALL nn and RST $FF */
    else if let OperandSet::Unary(ref op) = instruction.operand {
        cpu.sp -= 1;
        ram[cpu.sp as usize] = (cpu.pc & 0x00FF) as u8;
        cpu.sp -= 1;
        ram[cpu.sp as usize] = ((cpu.pc & 0xFF00) >> 8) as u8;
        cpu.pc = op.get_value(cpu, ram) as u16;
    }
    else {
        panic!("[ERROR] call/rst should have one or nonne operands");
    }
}

pub fn ret(instruction: &Instruction, cpu: &mut Cpu, ram: &mut [Byte;MEMORY_SIZE]){
    let pc = 
        (ram[(cpu.sp + 1) as usize] as u16) << 8 |
        (ram[(cpu.sp) as usize] as u16);
    
    if let OperandSet::Nullary = instruction.operand{
        cpu.sp += 2;
        cpu.pc = pc;
    }
    else if let OperandSet::Unary(ref op) = instruction.operand {
        if op.get_value(cpu, ram) == 0 {return;}
        cpu.sp += 2;
        cpu.pc = pc;
    }
    else {
        panic!("[ERROR] ret should have one or nonne operands");
    }
}

