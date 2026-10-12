use crate::core::{
    opcode::*,
    system::System
};

fn ld(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
        op1.update_system(op2.get_operand_value(system), system);
    } 
    else {
        panic!("[ERROR] LD Instructions should have two operands")
    }
}

fn inc(opcode: &Opcode, system: &mut System){
    if let OperandSet::Unary(ref op) = opcode.operand{
        let value = op.get_operand_value(system) + 1;
        op.update_system(value, system);
    }  
    else {
        panic!("[ERROR] INC Instructions should have one operands")
    }  
}

fn dec(opcode: &Opcode, system: &mut System){
    if let OperandSet::Unary(ref op) = opcode.operand{
        let value = op.get_operand_value(system) - 1;
        op.update_system(value, system);
    }  
    else {
        panic!("[ERROR] DEC Instructions should have one operands")
    }  
}

fn add(opcode: &Opcode, system: &mut System){
    if let OperandSet::Binary(ref op1, ref op2) = opcode.operand{
        op1.update_system(
            op1.get_operand_value(system) + op2.get_operand_value(system), 
            system
        );
    } 
    else {
        panic!("[ERROR] ADD Instructions should have two operands")
    } 
}

fn sub(opcode: &Opcode, system: &mut System){
    if let OperandSet::Unary(ref op) = opcode.operand{
        system.a -= op.get_operand_value(system) as u8;
    }  
    else {
        panic!("[ERROR] SUB Instructions should have one operands")
    } 
}