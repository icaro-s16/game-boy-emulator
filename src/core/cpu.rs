use crate::core::{
    handlers::{
        IME_DISABLED, 
        IME_ENABLED, 
        SHOULD_ENABLE_IME
    }, 
    imaps::*, 
    instruction::*, 
    system::*
};

const TIMA_ENABLE_BIT: u8 = 2; 
pub const DIV_ADDR: usize = 0xFF04;
pub const TIMA_ADDR: usize = 0xFF05;
pub const TMA_ADDR: usize = 0xFF06;
pub const TAC_ADDR: usize = 0xFF04;
const DIV_FALLING_EDGE: [u8;4] = [
    9,
    3,
    5,
    7
];

const VBLANK_INTERRUPT_BIT: u8 = 0;
const LCD_INTERRUPT_BIT: u8 = 1;
const TIMER_INTERRUPT_BIT: u8 = 2;
const SERIAL_INTERRUPT_BIT: u8 = 3;
const JOYPAD_INTERRUPT_BIT: u8 = 4;

const VBLANK_HANDLER_ADDR: u16 = 0x40;
const LCD_HANDLER_ADDR: u16 = 0x48;
const TIMER_HANDLER_ADDR: u16 = 0x50;
const SERIAL_HANDLER_ADDR: u16 = 0x58;
const JOYPAD_HANDLER_ADDR: u16 = 0x60;

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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum CpuState{
    Fetch,
    Decode,
    Interrupt,
    Execute
}

pub struct CpuContext{
    pub opcode: usize,
    pub instruction: Option<Instruction>,
    pub instruction_t_cycles: u8,
    pub instruction_tc_acca: u8,
    
    pub fedge: bool,
    pub tima_overflow: bool,
    pub tima_tc_acca: u8
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
    pub ime: u8,
    pub halt: bool,
    pub imaps: (
        [Instruction; 256], 
        [Option<Instruction>; 256]
    ),    

    pub state: CpuState,
    pub context: CpuContext
}

impl CpuContext{
    pub fn new() -> Self{
        Self { 
            opcode: 0, 
            instruction_t_cycles: 0, 
            instruction: None,
            instruction_tc_acca: 0,
            tima_overflow: false,
            fedge: false,
            tima_tc_acca: 0
        }
    }
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
            sp: 0xFFFE,
            pc: 0x0100,
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            ime: IME_DISABLED,
            halt: false,
            imaps: (
                prefixed_imap(), 
                unprefixed_imap()
            ),
            state: CpuState::Fetch,
            context: CpuContext::new()
        }
    }

    pub fn cycle(
        &mut self, 
        ram: &mut [Byte;MEMORY_SIZE]
    ){
        match self.state {
            CpuState::Fetch => {
                if !Cpu::sync(
                    &mut self.context.instruction_tc_acca, 
                    M_CYCLE
                ) {return;}
                
                let opcode_idx = ram[self.pc as usize] as usize; 
                self.pc += 1; 
                self.context.opcode = 
                    (self.context.opcode << 8) | opcode_idx;

                if self.context.opcode != 0xCB {
                    self.state = CpuState::Decode; 
                } 
            }
            CpuState::Interrupt => {
                if !Cpu::sync(
                    &mut self.context.instruction_tc_acca, 
                    M_CYCLE * 5
                ) {return;}

                self.sp -= 1;
                ram[self.sp as usize] = (self.pc & 0xFF) as u8;
                self.sp -= 1;
                ram[self.sp as usize] = (self.pc >> 8) as u8;  

                self.pc = 
                    if 
                        (ram[IE_ADDR] & VBLANK_INTERRUPT_BIT) != 0  &&
                        (ram[IF_ADDR] & VBLANK_INTERRUPT_BIT) != 0 
                    {
                        ram[IF_ADDR] &= !VBLANK_INTERRUPT_BIT;
                        VBLANK_HANDLER_ADDR 
                    }
                    else if 
                        (ram[IE_ADDR] & (1 << LCD_INTERRUPT_BIT)) != 0 &&
                        (ram[IF_ADDR] & (1 << LCD_INTERRUPT_BIT)) != 0 
                    {
                        ram[IF_ADDR] &= !(1 << LCD_INTERRUPT_BIT);
                        LCD_HANDLER_ADDR
                    }
                    else if 
                        (ram[IE_ADDR] & (1 << TIMER_INTERRUPT_BIT)) != 0 &&
                        (ram[IF_ADDR] & (1 << TIMER_INTERRUPT_BIT)) != 0
                    {
                        ram[IF_ADDR] &= !(1 << TIMER_INTERRUPT_BIT);
                        TIMER_HANDLER_ADDR
                    }
                    else if 
                        (ram[IE_ADDR] & (1 << SERIAL_INTERRUPT_BIT)) != 0 && 
                        (ram[IF_ADDR] & (1 << SERIAL_INTERRUPT_BIT)) != 0
                    {
                        ram[IF_ADDR] &= !(1 << SERIAL_INTERRUPT_BIT);
                        SERIAL_HANDLER_ADDR
                    }
                    else {
                        ram[IF_ADDR] &= !(1 << JOYPAD_INTERRUPT_BIT);
                        JOYPAD_HANDLER_ADDR
                    };
                
                self.ime = 0;
                self.state = CpuState::Fetch;

            }
            CpuState::Decode => {
                let instruction = 
                    if (self.context.opcode >> 8) == 0xCB {
                        self.imaps.0[self.context.opcode & 0xFF].clone()
                    }
                    else {
                        self.imaps.1[self.context.opcode & 0xFF].clone().unwrap()    
                    };
                    
                self.context.instruction_t_cycles = 
                    instruction
                    .get_t_cycles(self, ram);
                
                self.context.instruction = Some(instruction);

                self.context.opcode = 0;
                self.state = CpuState::Execute;
            }
            CpuState::Execute => {
                if !Cpu::sync(
                    &mut self.context.instruction_tc_acca, 
                    self.context.instruction_t_cycles  
                ) {return;}

                let instruction = 
                    self
                    .context.instruction
                    .take()
                    .unwrap();

                let should_enable_ime = 
                    self.ime == SHOULD_ENABLE_IME;

                (instruction.execute)(&instruction, self, ram);

                /* NOTE: This is used to apply the delay of 
                 * EI instructions, which enable the IME after 
                 * the execution of the instruction after EI
                 */
                if should_enable_ime
                {self.ime = IME_ENABLED;}

                self.state = 
                    if 
                        self.ime == IME_ENABLED && Cpu::active_interruption(ram) 
                    {
                        CpuState::Interrupt
                    }
                    else {
                        CpuState::Fetch
                    };
            }
        }
    }
    
    pub fn active_interruption(ram: &[Byte;MEMORY_SIZE]) -> bool {
        ((ram[IE_ADDR] & VBLANK_INTERRUPT_BIT) != 0 && (ram[IF_ADDR] & VBLANK_INTERRUPT_BIT) != 0) ||
        ((ram[IE_ADDR] & (1 << LCD_INTERRUPT_BIT)) != 0 && (ram[IF_ADDR] & (1 << LCD_INTERRUPT_BIT)) != 0) ||
        ((ram[IE_ADDR] & (1 << TIMER_INTERRUPT_BIT)) != 0 && (ram[IF_ADDR] & (1 << TIMER_INTERRUPT_BIT)) != 0) ||
        ((ram[IE_ADDR] & (1 << SERIAL_INTERRUPT_BIT)) != 0 && (ram[IF_ADDR] & (1 << SERIAL_INTERRUPT_BIT)) != 0) ||
        ((ram[IE_ADDR] & (1 << JOYPAD_INTERRUPT_BIT)) != 0 && (ram[IF_ADDR] & (1 << JOYPAD_INTERRUPT_BIT)) != 0)
    }

    pub fn timer(
        &mut self, 
        ram: &mut [Byte;MEMORY_SIZE],
    ){
        let mut div =   
            ((ram[DIV_ADDR] as u16) << 8) | ram[DIV_ADDR + 1] as u16;

        if div < 0xFFFF {
            div += 1;
        }
        else {
            div = 0;
        }
        
        ram[DIV_ADDR] =
            div as u8 >> 8;
        ram[DIV_ADDR + 1] = 
            div as u8 & 0xFF;

        /* NOTE: The increment rate of tima is controled
         * by falling edge of the last and operation, between
         * the div bit (which controle the increment rate)
         * with the Tac enable bit, and the current one.
         */
        let and_res = 
            (div & (1 << DIV_FALLING_EDGE[(ram[TAC_ADDR] & 0b11) as usize])) != 0 &&
            (ram[TAC_ADDR] & (1 << TIMA_ENABLE_BIT)) != 0;

        let tima_inc = 
            self.context.fedge && !and_res;
        
        self.context.fedge = and_res;

        if 
            !tima_inc || 
            (ram[TAC_ADDR] & (1 << TIMA_ENABLE_BIT)) == 0
        {return;} 

        if 
            self.context.tima_overflow &&
            Cpu::sync(
                &mut self.context.tima_tc_acca,
                M_CYCLE
            )
        {
            ram[TAC_ADDR] |= 1 << TIMA_ENABLE_BIT;
            ram[TIMA_ADDR] = ram[TMA_ADDR];
        }
        else if ram[TIMA_ADDR] < 0xFF {
            ram[TIMA_ADDR] += 1;
        }
        else{
            self.context.tima_overflow = true;
            ram[TIMA_ADDR] = 0;
        }
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
