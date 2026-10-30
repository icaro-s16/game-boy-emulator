use crate::core::{
    cpu::*, 
    joypad::*, 
    memory::Memory
};
use std::time::Instant;

pub type Byte = u8;
pub type Word = u16;

pub const M_CYCLE: u8 = 4; 
pub const MEMORY_SIZE: usize = 32 * 1_024;

const MASTER_CLOCK_HZ: u32 = 4_194_304;

const T_CYCLE_DELAY: f64 = 1.0 / MASTER_CLOCK_HZ as f64;

pub struct System{
    pub memory: Memory,
    pub cpu: Cpu,
    pub joypad: JoyPad
    // TODO: Ppu,
    // TODO: Apu,
}

impl System{
    pub fn new() -> Self{
        Self {
            memory: Memory::new(),
            joypad: JoyPad::new(kmap, sdl),
            cpu: Cpu::new()
        }
    }

    pub fn run(){
        let mut system = Self::new();
        let mut now = Instant::now();
        let mut delta = 0.0;
        let quit = false;


        while !quit {

            delta += now.elapsed().as_secs_f64();
            now = Instant::now();

            if T_CYCLE_DELAY > delta {continue;}

            delta -= T_CYCLE_DELAY;

            
            system.cpu.timer(
                &mut system.memory
            );

            system.joypad.update(
                &mut system.memory
            );

            system.cpu.cycle(
                &mut system.memory
            );
        }

    }

}




