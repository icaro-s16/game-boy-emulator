use sdl2::EventPump;

use crate::core::{
    cartridge::dynamic_cartridge_build, 
    cpu::*, 
    joypad::*, 
    memory::Memory
};

extern crate sdl2;

use std::{
    time::Instant,
    path::Path
};

pub const DISPLAY_HEIGHT: u32 = 144;
pub const DISPLAY_WIDTH: u32 = 160;

pub type Byte = u8;
pub type Word = u16;

pub const M_CYCLE: u8 = 4; 
const MASTER_CLOCK_HZ: u32 = 4_194_304;
const T_CYCLE_DELAY: f64 = 1.0 / MASTER_CLOCK_HZ as f64;

pub struct System{
    pub memory: Memory,
    pub cpu: Cpu,
    pub joypad: JoyPad,
    // TODO: Ppu,
    // TODO: Apu,

}

impl System{
    pub fn new(
        rom: &Path,
        event_pump: EventPump
    ) -> Self{

        Self {
            memory: Memory::new(dynamic_cartridge_build(rom)),
            joypad: JoyPad::new(KeyMap::default(), event_pump),
            cpu: Cpu::new()
        }
    }

    pub fn run(
        rom: &Path,
        pixel_height: u32,
        pixel_width: u32
    ){
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = 
            video_subsystem
            .window(
                "GameBoyEmu",
                DISPLAY_WIDTH * pixel_width,
                DISPLAY_HEIGHT * pixel_height
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = 
            window
            .into_canvas()
            .build()
            .unwrap();

        let mut system = 
            Self::new(
                rom,
                sdl_context.event_pump().unwrap()
            );
        
        let mut now = Instant::now();
        let mut delta = 0.0;
        let mut quit = false;


        while !quit {
            quit = system.joypad.update(
                &mut system.memory
            );

            delta += now.elapsed().as_secs_f64();
            now = Instant::now();

            if T_CYCLE_DELAY > delta {continue;}

            delta -= T_CYCLE_DELAY;
            
            system.cpu.timer(
                &mut system.memory
            );

            system.cpu.cycle(
                &mut system.memory
            );
        }

    }

}




