use crate::core::{
    instruction::*, 
    system::*
};

const DIV_ADDR: usize = 0xFF04;
const TIMA_ADDR: usize = 0xFF05;
const TMA_ADDR: usize = 0xFF06;
const TAC_ADDR: usize = 0xFF04;

const IE_ADDR: usize = 0xFFFF;
const IF_ADDR: usize = 0xFF0F;

#[allow(dead_code)]
#[derive(Clone)]
pub enum Register{
    Union(Box<Register>, Box<Register>),
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    PC,
    SP
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Flag{
    Z,
    NZ,
    C,
    NC,
    N,
    H
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CpuState{
    Fetch,
    Decode,
    Execute
}

pub struct Cpu{
    pub sp: Word,
    pub pc: Word,
    pub a: Byte,
    pub f: Byte,
    pub b: Byte,
    pub c: Byte,
    pub d: Byte,
    pub e: Byte,
    pub h: Byte,
    pub l: Byte,
    pub ime: bool,

    pub instruction_map: ([Instruction; 256], [Option<Instruction>; 256]),
    pub cpu_state: CpuState,
    pub current_opcode: usize,
    pub current_instruction: Option<Instruction>
}

impl Cpu{
    pub fn sync(tc_acca: &mut u8, expected_tc: u8) -> bool{
        if *tc_acca < expected_tc {
            *tc_acca += 1;
            false
        }
        else {
            *tc_acca -= expected_tc;
            true
        }
    }

    pub fn new() -> Self {
        Self { 
            sp: 0,
            pc: 0,
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            ime: false,
            instruction_map: (prefixed_imap(), unprefixed_imap()),
            cpu_state: CpuState::Fetch,
            current_opcode: 0,
            current_instruction: None
        }
    }

    pub fn cycle(
        &mut self, 
        ram: &mut [Byte;MEMORY_SIZE], 
        tc_acca: &mut u8
    ){
        match self.cpu_state {
            CpuState::Fetch => {
                if !Cpu::sync(tc_acca, M_CYCLE) {return;}
                let opcode_idx = ram[self.pc as usize] as usize; 
                self.pc += 1; 
                self.current_opcode = 
                    (self.current_opcode << 8) | opcode_idx;
                self.cpu_state = CpuState::Decode;  
            }
            CpuState::Decode => {
                self.current_instruction = 
                    if (self.current_opcode >> 8) == 0xCB {
                        Some(self.instruction_map.0[self.current_opcode & 0xFF].clone())
                    }
                    else {
                        Some(self.instruction_map.1[self.current_opcode & 0xFF].clone().unwrap())    
                    };
                self.current_opcode = 0;
                self.cpu_state = CpuState::Execute;
            }
            CpuState::Execute => {
                let instruction = 
                    self.current_instruction.as_ref().unwrap().clone();
                
                if !Cpu::sync(
                    tc_acca, 
                    instruction.operand.get_t_cycles()
                ) {return;}

                (instruction.execute)(&instruction, self, ram);

                self.handle_interrupts(ram);

                self.handle_timers();

                self.cpu_state = CpuState::Fetch;
            }
        }
    }

    pub fn handle_interrupts(
        &mut self, 
        ram: &[Byte;MEMORY_SIZE]
    ){
        if 
            self.ime && ram[IE_ADDR] != 0 && ram[IF_ADDR] != 0 
        {
            
        }
    }

    pub fn handle_timers(&mut self){

    }

    pub fn get_zflag(&self) -> bool{
        (self.f & (1 << 7)) != 0
    }

    pub fn set_zflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 7;
        }
        else {
            self.f &= !(1 << 7);
        }
    }

    pub fn get_nflag(&self) -> bool{
        (self.f & (1 << 6)) != 0
    }

    pub fn set_nflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 6;
        }
        else {
            self.f &= !(1 << 6);
        }
    }

    pub fn get_hflag(&self) -> bool{
        (self.f & (1 << 5)) != 0
    }

    pub fn set_hflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 5;
        }
        else {
            self.f &= !(1 << 5);
        }
    }

    pub fn get_cflag(&self) -> bool{
        (self.f & (1 << 4)) != 0
    }

    pub fn set_cflag(&mut self, value: bool){
        if value {
            self.f |= 1 << 4;
        }
        else {
            self.f &= !(1 << 4);
        }
    }
}
