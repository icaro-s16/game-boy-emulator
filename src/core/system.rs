use crate::core::cpu::*;
use std::time::Instant;

pub type Byte = u8;
pub type Word = u16;

pub const MEMORY_SIZE: usize = 32 * 1024;
const MASTER_CLOCK_HZ: u32 = 4_194_304;
const T_CYCLE_DELAY: f64 = 1.0 / (MASTER_CLOCK_HZ as f64 / 4.0);
pub const M_CYCLE: u8 = 4;   

pub struct System{
    pub cpu: Cpu,
    // pub ppu: Ppu,
    // pub apu: Apu,
}

impl System{
    pub fn new() -> Self{
        Self {
            cpu: Cpu::new()
        }
    }

    pub fn run(){
        let mut ram = [0;MEMORY_SIZE];
        let mut system = Self::new();
        let mut tc_acca: u8 = 0;
        let mut now = Instant::now();
        let mut delta = 0.0;
        let quit = false;

        while !quit {

            delta += now.elapsed().as_secs_f64();
            now = Instant::now();

            if T_CYCLE_DELAY > delta {continue;}

            delta -= T_CYCLE_DELAY;

            /* TODO: PPU T-Cycle pipeline
             * TODO: APU: T-Cycle pipeline
             */

            /* NOTE: The tc_acca is 
             * zeroed out, after 
             * complete an CPU state.
             */
            system.cpu.cycle(&mut ram, &mut tc_acca);

        }

    }

}




