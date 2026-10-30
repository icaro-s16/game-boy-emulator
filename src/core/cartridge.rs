use core::panic;
use std::{
    fs,
    path::Path
};

use crate::core::{
    memory::{
        ROM0_RANGE,
        ROMX_RANGE,
        SRAM_RANGE
    },
    system::Byte
};

const CTYPE_ADDR: usize = 0x0147;
const ROM_SIZE_ADDR: usize = 0x0148;
const RAM_SIZE_ADDR: usize = 0x0149;
const CHECKSUM_ADDR: usize = 0x014D;

pub trait Cartridge{
    fn index(&self, index: usize) -> &Byte;
    fn index_mut(&mut self, index: usize) -> &mut Byte;
    fn get_type(&self) -> u8;
}

struct RomOnly{
    rom0: [Byte; (ROM0_RANGE.1 + 1) - ROM0_RANGE.0],
    romx: [Byte; (ROMX_RANGE.1 + 1) - ROMX_RANGE.0],
    sram: [Byte; (SRAM_RANGE.1 + 1) - SRAM_RANGE.0]
}

impl RomOnly{
    const TYPE: u8 = 0x00;

    pub fn new(rom: &Path) -> Self{
        let rom: Vec<u8> = fs::read(rom).unwrap();
        
        const ROM0_SIZE: usize = (ROM0_RANGE.1 + 1) - ROM0_RANGE.0;
        const ROMX_SIZE: usize = (ROMX_RANGE.1 + 1) - ROMX_RANGE.0;
        const SRAM_SIZE: usize = (SRAM_RANGE.1 + 1) - SRAM_RANGE.0;
        
        let mut rom0: [Byte; ROM0_SIZE] = [0; ROM0_SIZE];
        let mut romx: [Byte; ROMX_SIZE] = [0; ROMX_SIZE];
        let mut sram: [Byte; SRAM_SIZE] = [0; SRAM_SIZE];

        for index in 0..ROM0_SIZE {
            if index >= rom.len() {break;}
            rom0[index] = rom[index];
        }

        for index in ROM0_SIZE..(ROM0_SIZE + ROMX_SIZE) {
            if index >= rom.len() {break;}
            romx[index] = rom[index];
        }

        for index in (ROM0_SIZE + ROMX_SIZE)..(ROM0_SIZE + ROMX_SIZE + SRAM_SIZE) {
            if index >= rom.len() {break;}
            sram[index] = rom[index];
        }

        Self{
            rom0,
            romx,
            sram
        }
    }
}

impl Cartridge for RomOnly {
    fn get_type(&self) -> u8 {
        RomOnly::TYPE
    }

    fn index(&self, index: usize) -> &Byte {
        if index >= ROM0_RANGE.0 && index <= ROM0_RANGE.1 {
            &self.rom0[index]
        }
        else if index >= ROMX_RANGE.0 && index <= ROMX_RANGE.1 {
            &self.romx[index]
        }
        else if index >= SRAM_RANGE.0 && index <= SRAM_RANGE.1 {
            &self.sram[index]
        }
        else {
            panic!("[ERROR] Index out of range")
        }
    }

    fn index_mut(&mut self, index: usize) -> &mut Byte {
        if index >= ROM0_RANGE.0 && index <= ROM0_RANGE.1 {
            &mut self.rom0[index]
        }
        else if index >= ROMX_RANGE.0 && index <= ROMX_RANGE.1 {
            &mut self.romx[index]
        }
        else if index >= SRAM_RANGE.0 && index <= SRAM_RANGE.1 {
            &mut self.sram[index]
        }
        else {
            panic!("[ERROR] Index out of range")
        }
    }
}


