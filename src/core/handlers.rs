use core::panic;

use crate::core::{
    cpu::*, instruction::*, memory::Memory, system::{
        Byte, 
        MEMORY_SIZE
    }
};
  
/* TODO: HALT
 * TODO: STOP
 * DONE: EI 
 * DONE: DAA
 * TODO: RETI
 * DONE: DI
 */

const HC_8B_MASK: usize = 0xF;
const HC_16B_MASK: usize = 0xFF;
const C_8B_MASK: usize = 0xFF;
const C_16B_MAKS: usize = 0xFFFF;

pub fn halt(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands {
        cpu.halt = HaltState::ENABLED;
        return;
    }

    panic!("[ERROR] halt should not have operands")
}

pub fn stop(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Unary(_) = instruction.operands {
        cpu.stop = StopState::DISABLED;
        memory[DIV_ADDR] = 0;
        memory[DIV_ADDR + 1] = 0;
        return;
    }
    panic!("[ERROR] stop should have one operand")
}

pub fn daa(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        let mut offset = 0;

        let cflag = cpu.get_cflag();
        let hcflag = cpu.get_hflag();
        let nflag = cpu.get_nflag();

        let mut should_carry = false;

        /* NOTE: To convert a lower nibble from hex 
        * to BCD format (where each nibble cannot 
        * exceed 0x09), we just need to add or 
        * subtract 0x06.
        * 
        * (0x06 accounts for the 6 hex values 
        * that are greater than 9, i.e., A through F).
        * 
        * Example: 0x07 + 0x07 = 0x0E; 0x0E + 0x06 = 0x14
        */
        if 
            !nflag && (cpu.a & 0x0F) > 0x09 ||
            hcflag
        {
            offset = 0x06;
        }

        if  
            !nflag && cpu.a > 0x99 ||
            cflag
        {
            offset |= 0x60;
            should_carry = true;
        }

        if nflag {
            cpu.a = ((cpu.a as u16 + offset) & 0xFF) as u8 ;  
        }
        else {
            cpu.a = ((cpu.a as u16 - offset) & 0xFF) as u8 ;
        }

        cpu.set_zflag(cpu.a == 0);
        cpu.set_hflag(false);
        cpu.set_cflag(should_carry);
        return; 
    }
    panic!("[ERROR] daa should not have operands");  
}

pub fn di(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        cpu.ime = ImeState::DISABLED;
        return; 
    }
    panic!("[ERROR] di should not have operands");  
}

pub fn ei(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        cpu.ime = ImeState::SCHEDULED;
        return; 
    }
    panic!("[ERROR] ei should not have operands");  
}

pub fn nop(instruction: &Instruction, _cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { return; }
    panic!("[ERROR] nop should not have operands");
}

pub fn ld(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) =
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] ld should have two operands");
        };

    let opcode = instruction.opcode;
    
    match opcode{
        /* LD HL, SP + e8 */
        0xF8 => {
            let result = op2.get_value(cpu, memory) + cpu.sp as usize;
            let hc_result = (op2.get_value(cpu, memory) & HC_8B_MASK) + (cpu.sp as usize & HC_8B_MASK); 
            op1.update(
                result, 
                cpu,
                memory
            );
            
            cpu.set_zflag(false);
            cpu.set_nflag(false);
            cpu.set_hflag(hc_result > HC_8B_MASK);
            cpu.set_cflag(result > C_8B_MASK);
        }
        /* LD A, [HL-] | LD A, [HL+] */
        0x3A | 0x2A => {
            op1.update(
                op2.get_value(cpu, memory), 
                cpu,
                memory
            );

            op2.update(
                if opcode == 0x3A{op2.get_value(cpu, memory) - 1} 
                else {op2.get_value(cpu, memory) + 1},
                cpu,
                memory
            );
        }
        /* LD [HL-], A | LD [HL+], A */
        0x32 | 0x22=> {
            op1.update(
                op2.get_value(cpu, memory), 
                cpu,
                memory
            );

            op1.update(
                if opcode == 0x3A{op1.get_value(cpu, memory) - 1} 
                else {op1.get_value(cpu, memory) + 1},
                cpu,
                memory
            );
        }
        _ => {
            op1.update(
                op2.get_value(cpu, memory), 
                cpu,
                memory
            );
        }
    }
    
}

pub fn ldh(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
        if let Some(address) = op2.as_any().downcast_ref::<Address>() {
            op1.update(
                address.get_value_with_offset(
                    0xFF, 
                    cpu,
                    memory
                ), 
                cpu,
                memory
            );
        }
        else if let Some(address) = op1.as_any().downcast_ref::<Address>() {
            address.update_with_offset(
                op2.get_value(cpu, memory),
                0xFF, 
                cpu,
                memory
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

pub fn push(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] push should have one operands");
        };

    let bytes = (operands.get_value(cpu, memory) as u16).to_be_bytes();
    cpu.sp -= 1;
    memory[cpu.sp as usize] = bytes[1];
    cpu.sp -= 1;
    memory[cpu.sp as usize] = bytes[0];

}

pub fn pop(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] pop should have one operands");
        };
    
    let sp_hbyte = memory[cpu.sp as usize];
    cpu.sp += 1;
    let sp_lbyte = memory[cpu.sp as usize];
    cpu.sp += 1;

    operands.update(
        u16::from_be_bytes([sp_hbyte, sp_lbyte]) as usize, 
        cpu,
        memory
    );
}

/* NOTE: This function is used with ADC and ADD */
pub fn add(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if 
        let OperandSet::Binary(ref op1, ref op2) = instruction.operands &&
        (!is_16bit_operand(op1.as_ref()) && !is_16bit_operand(op2.as_ref())) 
    {
        let mut result = op1.get_value(cpu, memory) + op2.get_value(cpu, memory);
        let mut hc_result = (op1.get_value(cpu, memory) & HC_8B_MASK) + (op2.get_value(cpu, memory) & HC_8B_MASK); 
        
        /* ADC instructions */
        if 
            instruction.opcode == 0x88 ||
            instruction.opcode == 0x8E ||
            instruction.opcode == 0xCE 
        {
            result += cpu.get_cflag() as usize;
            hc_result += cpu.get_cflag() as usize; 
        }

        op1.update(result, cpu, memory);

        cpu.set_zflag(result == 0);
        cpu.set_nflag(false);
        cpu.set_hflag(hc_result > HC_8B_MASK);
        cpu.set_cflag(result > C_8B_MASK);
    }
    else if let OperandSet::Binary(ref op1, ref op2) = instruction.operands{
        let result = op1.get_value(cpu, memory) + op2.get_value(cpu, memory);
        
        let (c_mask, hc_mask) = 
            if instruction.opcode == 0xE8 {
                cpu.set_zflag(false);
                (C_16B_MAKS, HC_16B_MASK)
            }
            else{
                (C_8B_MASK, HC_8B_MASK)
            };

        let hc_result= (op1.get_value(cpu, memory) & hc_mask) + (op2.get_value(cpu, memory) & hc_mask);

        op1.update(result, cpu, memory);
        cpu.set_nflag(false);
        cpu.set_hflag(hc_result > hc_mask);
        cpu.set_cflag(result > c_mask);
        
    }
    else {
        panic!("[ERROR] add/adc instruction should have one or two operands");
    }
}

/* NOTE: This function is used with SBC and SUB */
pub fn sub(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] sub/sbc should have two operands");
        };
    
    let mut result = op1.get_value(cpu, memory) - op2.get_value(cpu, memory);
    let mut hc_result = (op1.get_value(cpu, memory) & HC_8B_MASK) - (op2.get_value(cpu, memory) & HC_8B_MASK);
    
    /* SBC instructions */
    if 
        instruction.opcode == 0x98 ||
        instruction.opcode == 0x9E ||
        instruction.opcode == 0xDE 
    {
        result -= cpu.get_cflag() as usize; 
        hc_result -= cpu.get_cflag() as usize;
    }

    op1.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(true);
    cpu.set_hflag(hc_result > HC_8B_MASK);
    cpu.set_cflag(result > C_8B_MASK);

}

pub fn cp(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] cp should have two operands");
        };
    
    let result = op1.get_value(cpu, memory) - op2.get_value(cpu, memory);
    let hc_result = (op1.get_value(cpu, memory) & HC_8B_MASK) - (op2.get_value(cpu, memory) & HC_8B_MASK);
    op1.update(result, cpu, memory);
    
    cpu.set_zflag(result == 0);
    cpu.set_nflag(true);
    cpu.set_hflag(hc_result > HC_8B_MASK);
    cpu.set_cflag(result > C_8B_MASK);
}

pub fn inc(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands {
            op
        }
        else {
            panic!("[ERROR] inc should have one operands");
        };

    let hc_mask = 
        if is_16bit_operand(operands.as_ref()){
            HC_16B_MASK
        }
        else {
            HC_8B_MASK
        };
    
    let result = operands.get_value(cpu, memory) + 1;
    let hc_result = (operands.get_value(cpu, memory) & hc_mask) + 1;

    operands.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(hc_result > hc_mask);
}

pub fn dec(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands {
            op
        }
        else {
            panic!("[ERROR] dec should have one operands");
        };
    
    let hc_mask = 
        if is_16bit_operand(operands.as_ref()){
            HC_16B_MASK
        }
        else {
            HC_8B_MASK
        };
    
    let result = operands.get_value(cpu, memory) - 1;
    let hc_result = (operands.get_value(cpu, memory) & hc_mask) - 1;

    operands.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(hc_result > hc_mask);
}

pub fn and(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] and should have two operands");
        };
    
    let result = op1.get_value(cpu, memory) & op2.get_value(cpu, memory);
    
    op1.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(true);
    cpu.set_cflag(false);
}

pub fn or(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] or should have two operands");
        };
    
    let result = op1.get_value(cpu, memory) | op2.get_value(cpu, memory);
    
    op1.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn xor(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
            (op1, op2)
        }
        else {
            panic!("[ERROR] xor should have two operands");
        };
    
    let result = op1.get_value(cpu, memory) ^ op2.get_value(cpu, memory);
    
    op1.update(result, cpu, memory);

    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn ccf(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        cpu.set_nflag(false);
        cpu.set_hflag(false);
        cpu.set_cflag(!cpu.get_cflag());
        return; 
    }
    panic!("[ERROR] ccf should not have operands");
}

pub fn scf(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        cpu.set_nflag(false);
        cpu.set_hflag(false);
        cpu.set_cflag(true);
        return; 
    }
    panic!("[ERROR] scf should not have operands");
}

pub fn cpl(instruction: &Instruction, cpu: &mut Cpu, _memory: &mut Memory){
    if let OperandSet::Nullary = instruction.operands { 
        cpu.a = !cpu.a;
        cpu.set_nflag(true);
        cpu.set_hflag(true);
        return; 
    }
    panic!("[ERROR] cpl should not have operands");
}

pub fn rlc(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] rlc should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] rlc can not be an 16 bit operands");
    }

    let b7 = ((operands.get_value(cpu, memory) & (1 << 7)) != 0) as u8;
    operands.update(
        (operands.get_value(cpu, memory) << 1) | b7 as usize, 
        cpu,
        memory
    );

    if instruction.opcode == 0x07{
        cpu.set_zflag(false);
    }
    else {
        let result = operands.get_value(cpu, memory);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn rl(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] rl should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] rl can not be an 16 bit operands");
    }

    let b7 = ((operands.get_value(cpu, memory) & (1 << 7)) != 0) as u8;
    operands.update(
        (operands.get_value(cpu, memory) << 1) | cpu.get_cflag() as usize, 
        cpu,
        memory
    );

    if instruction.opcode == 0x17{
        cpu.set_zflag(false);
    }
    else {
        let result = operands.get_value(cpu, memory);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn rrc(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] rrc should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] rrc can not be an 16 bit operands");
    }

    let b0 = (operands.get_value(cpu, memory) & 1) as u8;
    operands.update(
        (operands.get_value(cpu, memory) >> 1) | b0 as usize, 
        cpu,
        memory
    );

    if instruction.opcode == 0x0F{
        cpu.set_zflag(false);
    }
    else {
        let result = operands.get_value(cpu, memory);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn rr(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] rr should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] rr can not be an 16 bit operands");
    }

    let b0 = (operands.get_value(cpu, memory) & 1) as u8;
    operands.update(
        (operands.get_value(cpu, memory) >> 1) | cpu.get_cflag() as usize, 
        cpu,
        memory
    );

    if instruction.opcode == 0x1F{
        cpu.set_zflag(false);
    }
    else {
        let result = operands.get_value(cpu, memory);
        cpu.set_zflag(result == 0);
    }
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn sla(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] sla should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] sla can not be an 16 bit operands");
    }

    let b7 = ((operands.get_value(cpu, memory) & (1 << 7)) != 0) as u8;
    operands.update(
        operands.get_value(cpu, memory) << 1, 
        cpu,
        memory
    );

    let result = operands.get_value(cpu, memory);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b7 != 0);
}

pub fn sra(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] sra should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] sra can not be an 16 bit operands");
    }

    let b0 = (operands.get_value(cpu, memory) & 1) as u8;
    let b7 = ((operands.get_value(cpu, memory) & (1 << 7)) != 0) as u8;
    operands.update(
        (operands.get_value(cpu, memory) >> 1) | (b7 << 7) as usize, 
        cpu,
        memory
    );

    let result = operands.get_value(cpu, memory);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn swap(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] swap should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] swap can not be an 16 bit operands");
    }

    operands.update(
        (operands.get_value(cpu, memory) & 0x0F) | (operands.get_value(cpu, memory) & 0xF0), 
        cpu,
        memory
    );

    let result = operands.get_value(cpu, memory);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(false);
}

pub fn srl(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let operands = 
        if let OperandSet::Unary(ref op) = instruction.operands{
            op
        }
        else {
            panic!("[ERROR] srl should have one operands");
        };
    
    if is_16bit_operand(operands.as_ref()) {
        panic!("[ERROR] srl can not be an 16 bit operands");
    }

    let b0 = (operands.get_value(cpu, memory) & 1) as u8;
    operands.update(
        operands.get_value(cpu, memory) >> 1, 
        cpu,
        memory
    );

    let result = operands.get_value(cpu, memory);
    cpu.set_zflag(result == 0);
    cpu.set_nflag(false);
    cpu.set_hflag(false);
    cpu.set_cflag(b0 != 0);
}

pub fn bit(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands{
            (op1.get_value(cpu, memory), op2.get_value(cpu, memory))
        }
        else {
            panic!("[ERROR] bit should have two operands");
        };

    cpu.set_zflag((op2 & (1 << op1)) != 0);
    cpu.set_nflag(false);
    cpu.set_hflag(true);
}

pub fn res(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) =  
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands{
            (op1.get_value(cpu, memory), op2)
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
        op2.get_value(cpu, memory) & !(1 << op1),
        cpu,
        memory
    );
}

pub fn set(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let (op1, op2) = 
        if let OperandSet::Binary(ref op1, ref op2) = instruction.operands{
            (op1.get_value(cpu, memory), op2)
        }
        else {
            panic!("[ERROR] set should have two operands");
        };

    op2.update(
        op2.get_value(cpu, memory) | (1 << op1),
        cpu,
        memory
    );
}

pub fn jp(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
        if op1.get_value(cpu, memory) == 0 {return;}
        cpu.pc = op2.get_value(cpu, memory) as u16;
    }
    else if let OperandSet::Unary(ref op) = instruction.operands {
        cpu.pc = op.get_value(cpu, memory) as u16;
    }
    else {
        panic!("[ERROR] jp should have one or nonne operands");
    }
}

pub fn jr(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
        if op1.get_value(cpu, memory) == 0 {return;}
        cpu.pc += op2.get_value(cpu, memory) as u16;
    }
    else if let OperandSet::Unary(ref op) = instruction.operands {
        cpu.pc += op.get_value(cpu, memory) as u16;
    }
    else {
        panic!("[ERROR] jr should have one or nonne operands");
    }
}

pub fn call(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    if let OperandSet::Binary(ref op1, ref op2) = instruction.operands {
        if op1.get_value(cpu, memory) == 0 {return;}
        cpu.sp -= 1;
        memory[cpu.sp as usize] = (cpu.pc & 0xFF) as u8;
        cpu.sp -= 1;
        memory[cpu.sp as usize] = (cpu.pc >> 8) as u8;
        cpu.pc = op2.get_value(cpu, memory) as u16;
    }
    /* NOTE: Can be used in CALL nn and RST $FF */
    else if let OperandSet::Unary(ref op) = instruction.operands {
        cpu.sp -= 1;
        memory[cpu.sp as usize] = (cpu.pc & 0xFF) as u8;
        cpu.sp -= 1;
        memory[cpu.sp as usize] = (cpu.pc >> 8) as u8;
        cpu.pc = op.get_value(cpu, memory) as u16;
    }
    else {
        panic!("[ERROR] call/rst should have one or nonne operands");
    }
}

pub fn ret(instruction: &Instruction, cpu: &mut Cpu, memory: &mut Memory){
    let pc = 
        (memory[(cpu.sp + 1) as usize] as u16) << 8 |
        (memory[(cpu.sp) as usize] as u16);
    
    if let OperandSet::Nullary = instruction.operands{
        
        /* NOTE: Different from EI and DI
         * RETI dosent have the IME 
         * activation delay.
         */
        if instruction.opcode == 0xD9 {
            cpu.ime = ImeState::ENABLED;
        }
        cpu.sp += 2;
        cpu.pc = pc;
    }
    else if let OperandSet::Unary(ref op) = instruction.operands {
        if op.get_value(cpu, memory) == 0 {return;}
        cpu.sp += 2;
        cpu.pc = pc;
    }
    else {
        panic!("[ERROR] ret/reti should have one or nonne operands");
    }
}

